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
    /// Texture id of a 1x1 white texture used to tint overlay quads.
    highlight_texture_id: usize,
    /// Latest cursor position in physical pixels, or `None` when the cursor is
    /// outside the window.
    cursor_pos: Option<(f64, f64)>,
    /// Grid coordinates of the tile currently under the cursor, if any.
    hovered_tile: Option<(i32, i32)>,
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
        let mut renderer = crate::renderer::Renderer::new(&device, &queue, &config, &camera);
        let batcher = crate::renderer::batch::SpriteBatcher::new();

        // Register a 1x1 white texture so overlay quads (the hover highlight)
        // are coloured purely by their vertex color.
        let white_pixel = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([255, 255, 255, 255]),
        ));
        let white_texture =
            crate::renderer::texture::Texture::from_image(&device, &queue, &white_pixel, Some("white"))
                .expect("1x1 white texture is always valid");
        let highlight_texture_id = renderer.register_texture(&device, &white_texture);

        // Prefer a hand-authored map from JSON; fall back to procedural
        // generation if the file is missing or invalid so the game always
        // boots with *something* to render.
        const MAP_PATH: &str = "assets/maps/test_map.json";
        let map = match crate::game::map::load_map_from_file(MAP_PATH) {
            Ok(map) => {
                log::info!("Loaded map from {MAP_PATH} ({}x{})", map.width, map.height);
                map
            }
            Err(e) => {
                log::warn!("Failed to load map from {MAP_PATH}: {e}. Falling back to procedural island.");
                crate::game::map::Map::generate_island(20, 20, 1337)
            }
        };

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
            highlight_texture_id,
            cursor_pos: None,
            hovered_tile: None,
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

        // Resolve which tile the cursor is over (camera may have moved, so this
        // is recomputed every frame).
        self.hovered_tile = self.pick_tile();

        // Clear the dynamic batcher for the new frame. Static terrain lives in
        // `terrain_batcher` and is NOT rebuilt here — only dynamic objects
        // (towers, enemies, projectiles) are queued each frame.
        self.batcher.clear();

        // Draw a translucent 3D highlight box around the hovered tile's top
        // block, at its real height. The same `add_highlight_box` call will
        // later work for towers floating above the ground.
        if let Some((gx, gz)) = self.hovered_tile {
            if let Some(tile) = self.map.get_tile(gx, gz) {
                let pos_x = (gx as f32 - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
                let pos_z = (gz as f32 - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
                // Centre of the tile's topmost cube.
                let center_y = tile.grid_y as f32 * TILE_WORLD_SIZE;

                // Inflate the shell slightly so it wraps just outside the block
                // and doesn't z-fight with the terrain faces.
                let size = glam::Vec3::splat(TILE_WORLD_SIZE + HIGHLIGHT_INFLATE);

                self.batcher.add_highlight_box(
                    glam::Vec3::new(pos_x, center_y, pos_z),
                    size,
                    HIGHLIGHT_COLOR,
                    self.highlight_texture_id,
                );
            }
        }

        // TODO: queue dynamic objects (towers, enemies, projectiles) here via
        // self.batcher.add_sprite(...) / add_cube(...).

        // Compile batches and upload to the GPU.
        self.batcher.finalize(&self.device, &self.queue);
    }

    /// Records the latest cursor position (physical pixels).
    pub fn set_cursor_position(&mut self, x: f64, y: f64) {
        self.cursor_pos = Some((x, y));
    }

    /// Clears the cursor position (e.g. when it leaves the window) so no tile
    /// is highlighted.
    pub fn clear_cursor(&mut self) {
        self.cursor_pos = None;
        self.hovered_tile = None;
    }

    /// Returns the grid coordinates of the tile currently under the cursor, or
    /// `None` if the cursor is off-screen or misses the terrain.
    ///
    /// Casts the cursor ray against every tile's full 3D column (from the
    /// terrain base up to the tile's top) and returns the nearest hit. Because
    /// it tests the whole box, hovering a vertical side face — a raised rock
    /// cube, a cliff between heights, or the dirt border — picks that tile too,
    /// not just the flat tops.
    fn pick_tile(&self) -> Option<(i32, i32)> {
        let (px, py) = self.cursor_pos?;
        let (w, h) = (self.config.width, self.config.height);
        let (origin, dir) = self.camera.screen_ray(px as f32, py as f32, w, h)?;

        let half = TILE_WORLD_SIZE / 2.0;
        let column_bottom = TERRAIN_BASE_LEVEL as f32 * TILE_WORLD_SIZE - half;
        let (mw, mh) = (self.map.width, self.map.height);

        let mut best: Option<(f32, (i32, i32))> = None;

        for gx in 0..mw {
            for gz in 0..mh {
                let tile = match self.map.get_tile(gx, gz) {
                    Some(t) => t,
                    None => continue,
                };

                let pos_x = (gx as f32 - mw as f32 / 2.0) * TILE_WORLD_SIZE;
                let pos_z = (gz as f32 - mh as f32 / 2.0) * TILE_WORLD_SIZE;
                let top = tile.grid_y as f32 * TILE_WORLD_SIZE + half;

                let min = glam::Vec3::new(pos_x - half, column_bottom, pos_z - half);
                let max = glam::Vec3::new(pos_x + half, top, pos_z + half);

                if let Some(t) = ray_aabb(origin, dir, min, max) {
                    if best.map_or(true, |(best_t, _)| t < best_t) {
                        best = Some((t, (gx, gz)));
                    }
                }
            }
        }

        best.map(|(_, tile)| tile)
    }

    /// The tile currently under the cursor, if any. Useful for gameplay
    /// (placing towers, inspecting tiles, etc.).
    pub fn hovered_tile(&self) -> Option<(i32, i32)> {
        self.hovered_tile
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

/// World-space edge length of a single terrain tile/cube. Terrain meshing and
/// cursor picking both use this so they stay in sync.
pub const TILE_WORLD_SIZE: f32 = 0.12;

/// Lowest cube level drawn for the terrain skirt (border depth). Meshing and
/// picking share this so a tile's pickable column matches its visible extent.
const TERRAIN_BASE_LEVEL: i32 = -2;

/// Translucent warm-yellow tint for the hovered-tile highlight overlay.
const HIGHLIGHT_COLOR: [f32; 4] = [1.0, 0.9, 0.3, 0.45];

/// How much larger the highlight shell is than the block it wraps, so it sits
/// just outside the terrain faces and avoids z-fighting.
const HIGHLIGHT_INFLATE: f32 = 0.006;

/// Ray vs axis-aligned box intersection (slab method).
///
/// Returns the distance along `dir` to the nearest entry point (clamped to 0
/// when the origin is already inside), or `None` if the ray misses the box.
/// `dir` is expected to be normalised.
fn ray_aabb(origin: glam::Vec3, dir: glam::Vec3, min: glam::Vec3, max: glam::Vec3) -> Option<f32> {
    let inv = glam::Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
    let t1 = (min - origin) * inv;
    let t2 = (max - origin) * inv;

    let t_enter = t1.min(t2).max_element();
    let t_exit = t1.max(t2).min_element();

    if t_exit >= t_enter.max(0.0) {
        Some(t_enter.max(0.0))
    } else {
        None
    }
}

/// Builds a face-culled terrain mesh from the map into `batcher`, then uploads
/// it to the GPU. Only visible faces are generated:
///
/// * the top face of every tile, and
/// * side faces only where a tile is taller than its neighbour (the exposed
///   "cliff"); faces touching an equal-or-taller neighbour are skipped.
///
/// Map-border tiles emit a short skirt down to `TERRAIN_BASE_LEVEL` so the underside of
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

    let cube_size = glam::Vec3::splat(TILE_WORLD_SIZE);

    batcher.clear();

    let w = map.width;
    let h = map.height;

    for x in 0..w {
        for z in 0..h {
            let tile = match map.get_tile(x, z) {
                Some(t) => t,
                None => continue,
            };

            let pos_x = (x as f32 - w as f32 / 2.0) * TILE_WORLD_SIZE;
            let pos_z = (z as f32 - h as f32 / 2.0) * TILE_WORLD_SIZE;
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
                let neighbour_h = map.get_tile(nx, nz).map(|t| t.grid_y).unwrap_or(TERRAIN_BASE_LEVEL);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_aabb_hits_box_in_front() {
        // Ray starting at origin pointing +Z into a unit box on the +Z axis.
        let origin = glam::Vec3::new(0.0, 0.0, 0.0);
        let dir = glam::Vec3::new(0.0, 0.0, 1.0);
        let min = glam::Vec3::new(-0.5, -0.5, 2.0);
        let max = glam::Vec3::new(0.5, 0.5, 3.0);

        let t = ray_aabb(origin, dir, min, max).expect("ray should hit the box");
        assert!((t - 2.0).abs() < 1e-5, "entry distance should be 2.0, got {t}");
    }

    #[test]
    fn ray_aabb_misses_box_to_the_side() {
        let origin = glam::Vec3::new(0.0, 0.0, 0.0);
        let dir = glam::Vec3::new(0.0, 0.0, 1.0);
        // Box is offset in X so the +Z ray never enters it.
        let min = glam::Vec3::new(5.0, -0.5, 2.0);
        let max = glam::Vec3::new(6.0, 0.5, 3.0);

        assert!(ray_aabb(origin, dir, min, max).is_none());
    }

    #[test]
    fn ray_aabb_behind_is_missed() {
        // Box is entirely behind the ray origin (negative Z), ray points +Z.
        let origin = glam::Vec3::new(0.0, 0.0, 0.0);
        let dir = glam::Vec3::new(0.0, 0.0, 1.0);
        let min = glam::Vec3::new(-0.5, -0.5, -3.0);
        let max = glam::Vec3::new(0.5, 0.5, -2.0);

        assert!(ray_aabb(origin, dir, min, max).is_none());
    }
}
