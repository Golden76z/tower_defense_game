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
    batcher: crate::renderer::batch::SpriteBatcher,
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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: true,
            renderer,
            batcher,
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
            self.renderer.resize(&self.config);
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

        // Clear batcher for the new frame
        self.batcher.clear();

        // Generate a 10x10 grid of 3D cubes forming an animated wave
        let grid_size = 10;
        for x in 0..grid_size {
            for z in 0..grid_size {
                let pos_x = (x as f32 - grid_size as f32 / 2.0) * 0.15;
                let pos_z = (z as f32 - grid_size as f32 / 2.0) * 0.15;
                
                // Animated heightmap using a combination of sine and cosine waves
                let height = ((x as f32 * 0.5 + self.time_elapsed).sin() 
                    + (z as f32 * 0.5 + self.time_elapsed).cos()) * 0.05;

                self.batcher.add_cube(
                    glam::Vec3::new(pos_x, height - 0.1, pos_z),
                    glam::Vec3::new(0.12, 0.12, 0.12),
                    0, // Uses texture ID 0 (our test/default texture)
                );
            }
        }

        // Add 5 orbiting vertical sprites above the grid
        let num_sprites = 5;
        for i in 0..num_sprites {
            let angle = (i as f32 / num_sprites as f32) * std::f32::consts::TAU + self.time_elapsed;
            let radius = 0.4;
            let pos = glam::Vec3::new(angle.cos() * radius, 0.25, angle.sin() * radius);
            
            let sprite = crate::renderer::sprite::Sprite::new(
                pos,
                glam::Vec2::new(0.08, 0.08),
                0,
                angle,
            );
            self.batcher.add_sprite(sprite, crate::renderer::sprite::SpriteAlignment::Vertical);
        }

        // Compile batches and upload to the GPU
        self.batcher.finalize(&self.device, &self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render_batch(&view, &self.device, &self.queue, self.color, &self.batcher)?;

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

// Removed Vertex implementation (moved to renderer module)
