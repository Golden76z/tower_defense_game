use std::sync::Arc;
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// This will store the state of our game
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    renderer: crate::renderer::Renderer,
    /// Per-frame batcher for dynamic objects (towers, enemies, projectiles).
    batcher: crate::renderer::batch::SpriteBatcher,
    /// Cached static terrain mesh, rebuilt only when the map changes.
    terrain_batcher: crate::renderer::batch::SpriteBatcher,
    map: crate::game::map::Map,
    pub window: Arc<Window>,
    pub color: wgpu::Color,
    pub time: crate::engine::time::Time,
    time_elapsed: f32,
    pub camera: crate::renderer::camera::Camera,
    pub camera_controller: crate::renderer::camera::CameraController,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY | wgpu::Backends::GL,

            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,

            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        // Request adapter - FIXED
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        println!("Using adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Configure the surface - THIS IS IMPORTANT!
        surface.configure(&device, &config);

        let camera = crate::renderer::camera::Camera::new();
        let camera_controller = crate::renderer::camera::CameraController::default();
        let renderer = crate::renderer::Renderer::new(&device, &queue, &config, &camera);
        let batcher = crate::renderer::batch::SpriteBatcher::new();

        let map = crate::game::map::Map::generate_island(20, 20, 1337);

        // Build the static terrain mesh once. It only needs rebuilding when
        // the map itself changes, not every frame.
        let mut terrain_batcher = crate::renderer::batch::SpriteBatcher::new();
        build_terrain_mesh(&map, &mut terrain_batcher, &device, &queue);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: true,
            renderer,
            batcher,
            terrain_batcher,
            map,
            window,
            color: wgpu::Color {
                r: 0.05,
                g: 0.05,
                b: 0.08,
                a: 1.0,
            },
            time: crate::engine::time::Time::new(),
            time_elapsed: 0.0,
            camera,
            camera_controller,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.renderer.resize(&self.device, &self.config);
            self.renderer.update_camera_uniform(&self.queue, &self.camera, width, height);
        }
    }

    pub fn update(&mut self) {
        let delta = self.time.update();
        let dt = delta.as_secs_f32();
        self.time_elapsed += dt;

        // Update the camera and upload new uniform matrix to GPU
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.renderer.update_camera_uniform(&self.queue, &self.camera, self.config.width, self.config.height);

        // Clear the dynamic batcher for the new frame. Static terrain lives in
        // `terrain_batcher` and is NOT rebuilt here — only dynamic objects
        // (towers, enemies, projectiles) are queued each frame.
        self.batcher.clear();

        // TODO: queue dynamic objects (towers, enemies, projectiles) here via
        // self.batcher.add_sprite(...) / add_cube(...).

        // Compile batches and upload to the GPU.
        self.batcher.finalize(&self.device, &self.queue);
    }

    /// Rebuilds the cached terrain mesh from the current map. Call this after
    /// the map changes (e.g. a wall is placed or a tile is edited); it is not
    /// needed on every frame.
    pub fn rebuild_terrain(&mut self) {
        build_terrain_mesh(&self.map, &mut self.terrain_batcher, &self.device, &self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Draw the cached terrain first, then dynamic objects, in a single
        // pass. Depth testing resolves ordering between the two.
        self.renderer.render_batches(
            &view,
            &self.device,
            &self.queue,
            self.color,
            &[&self.terrain_batcher, &self.batcher],
        )?;

        output.present();

        Ok(())
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                self.camera_controller.process_key(*code, key_state.is_pressed())
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            _ => false,
        }
    }
}

/// Builds a face-culled terrain mesh from the map into `batcher`, then uploads
/// it to the GPU. Only visible faces are generated:
///
/// * the top face of every tile, and
/// * side faces only where a tile is taller than its neighbour (the exposed
///   "cliff"); faces touching an equal-or-taller neighbour are skipped.
///
/// Map-border tiles emit a short skirt down to `BASE_LEVEL` so the underside of
/// the island isn't visible from the isometric camera angle. This replaces the
/// old approach of stacking full cubes from a fixed `-4` foundation, which
/// generated huge amounts of hidden geometry every frame.
fn build_terrain_mesh(
    map: &crate::game::map::Map,
    batcher: &mut crate::renderer::batch::SpriteBatcher,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    use crate::game::map::tile::TileType;
    use crate::renderer::batch::Face;

    const CUBE_SPACING: f32 = 0.12;
    const BASE_LEVEL: i32 = -2; // Border skirt depth.
    let cube_size = glam::Vec3::new(0.12, 0.12, 0.12);

    batcher.clear();

    let w = map.width;
    let h = map.height;

    for x in 0..w {
        for z in 0..h {
            let tile = match map.get_tile(x, z) {
                Some(t) => t,
                None => continue,
            };

            let pos_x = (x as f32 - w as f32 / 2.0) * CUBE_SPACING;
            let pos_z = (z as f32 - h as f32 / 2.0) * CUBE_SPACING;
            let top_h = tile.grid_y;

            // Texture IDs: 0 = grass side, 1 = grass top, 2 = water, 3 = sand, 4 = rock.
            let (top_tex, side_tex) = match tile.tile_type {
                TileType::Grass => (1usize, 0usize),
                TileType::Water => (2, 2),
                TileType::Sand => (3, 3),
                TileType::Rock => (4, 4),
                TileType::Path => (3, 0), // Use sand for path for now.
            };

            // Top face — always visible from above.
            batcher.add_face(
                glam::Vec3::new(pos_x, top_h as f32 * cube_size.y, pos_z),
                cube_size,
                Face::Top,
                top_tex,
            );

            // Exposed side skirts: one face per level between the neighbour's
            // top and ours. Levels at or below the neighbour are hidden.
            let neighbours = [
                (x + 1, z, Face::East),
                (x - 1, z, Face::West),
                (x, z + 1, Face::South),
                (x, z - 1, Face::North),
            ];

            for (nx, nz, face) in neighbours {
                let neighbour_h = map.get_tile(nx, nz).map(|t| t.grid_y).unwrap_or(BASE_LEVEL);

                let mut level = neighbour_h + 1;
                while level <= top_h {
                    batcher.add_face(
                        glam::Vec3::new(pos_x, level as f32 * cube_size.y, pos_z),
                        cube_size,
                        face,
                        side_tex,
                    );
                    level += 1;
                }
            }
        }
    }

    batcher.finalize(device, queue);
}

// Removed Vertex implementation (moved to renderer module)
