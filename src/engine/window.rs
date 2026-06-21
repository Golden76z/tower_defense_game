use std::sync::Arc;
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct GoldPopup {
    pub position: glam::Vec3,
    pub lifetime: f32,
    pub amount: u32,
}

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
    enemy_manager: crate::game::enemies::EnemyManager,
    /// 3D model for the enemy.
    enemy_model: crate::renderer::model::Model,
    /// Texture id for the enemy sprite.
    _enemy_texture_id: usize,
    /// Texture id of a 1x1 white texture used to tint overlay quads.
    highlight_texture_id: usize,
    pub tower_manager: crate::game::towers::manager::TowerManager,
    /// 3D model for the tower.
    tower_model: crate::renderer::model::Model,
    /// Active projectiles flying towards enemies.
    projectiles: Vec<crate::game::projectiles::Projectile>,
    /// 3D model for the projectile.
    projectile_model: crate::renderer::model::Model,
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
    /// Centralised keyboard/mouse state for game logic to query.
    input: crate::engine::input::InputState,
    /// When true, the enemy path is drawn as ground markers (toggle: `P`).
    show_path_debug: bool,
    /// Economy tracking player money.
    pub economy: crate::game::economy::Economy,
    /// Player stats including lives.
    pub player_stats: crate::game::player_stats::PlayerStats,
    /// Feedback status message shown in the window title.
    pub placement_status: String,
    /// Gold reward popup sprites.
    popups: Vec<GoldPopup>,
    /// Texture id for the gold popup sprite.
    popup_texture_id: usize,
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

        // Load the enemy texture
        let enemy_image = image::open("assets/sprites/enemy.png")
            .expect("Failed to load enemy texture")
            .to_rgba8();
        let enemy_texture = crate::renderer::texture::Texture::from_image(
            &device,
            &queue,
            &image::DynamicImage::ImageRgba8(enemy_image),
            Some("enemy"),
        )
        .expect("enemy texture should be valid");
        let enemy_texture_id = renderer.register_texture(&device, &enemy_texture);

        // Generate and register gold popup texture
        let popup_image = create_gold_texture("+10");
        let popup_texture = crate::renderer::texture::Texture::from_image(
            &device,
            &queue,
            &image::DynamicImage::ImageRgba8(popup_image),
            Some("gold_popup"),
        )
        .expect("gold popup texture should be valid");
        let popup_texture_id = renderer.register_texture(&device, &popup_texture);

        // Load 3D model for the tower
        let tower_model = crate::renderer::model::Model::load("assets/models/tower.obj")
            .expect("Failed to load tower.obj model");

        // Load 3D model for projectile
        let projectile_model = crate::renderer::model::Model::load("assets/models/projectile.obj")
            .expect("Failed to load projectile.obj model");

        // Load 3D model for enemies
        let enemy_model = crate::renderer::model::Model::load("assets/models/enemy.obj")
            .expect("Failed to load enemy.obj model");

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

        let mut enemy_manager = crate::game::enemies::EnemyManager::new();
        
        // Spawn test enemies if we have a path
        if let Some(start_pos) = map.path.start() {
            let enemy = crate::game::enemies::basic_enemy::BasicEnemy::new(start_pos);
            enemy_manager.spawn_enemy(Box::new(enemy));
        }

        let state = Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: true,
            renderer,
            batcher,
            terrain_batcher,
            map,
            enemy_manager,
            enemy_model,
            _enemy_texture_id: enemy_texture_id,
            highlight_texture_id,
            tower_model,
            tower_manager: crate::game::towers::manager::TowerManager::new(),
            projectiles: Vec::new(),
            projectile_model,
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
            input: crate::engine::input::InputState::new(),
            show_path_debug: false,
            economy: crate::game::economy::Economy::new(200),
            player_stats: crate::game::player_stats::PlayerStats::new(5),
            placement_status: "Ready".to_string(),
            popups: Vec::new(),
            popup_texture_id,
        };

        state.update_window_title();

        Ok(state)
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

    pub fn update_window_title(&self) {
        let title = format!(
            "Isoguard - Lives: {}/{} | Gold: {} | Basic Tower Cost: {} | Status: {}",
            self.player_stats.lives,
            self.player_stats.max_lives,
            self.economy.money,
            crate::game::towers::manager::TowerType::Basic.cost(),
            self.placement_status
        );
        self.window.set_title(&title);
    }

    /// Advances the game one frame.
    ///
    /// Logic runs on a fixed timestep (so it's deterministic and rate-stable),
    /// while rendering preparation runs once per frame at whatever rate the
    /// host draws. See [`crate::engine::time::Time`].
    pub fn update(&mut self) {
        // Measure real elapsed time and fold it into the accumulator.
        self.time.begin_frame();

        // Drain the accumulator in fixed steps. Zero, one, or several logic
        // updates may run per frame depending on render rate; the spiral-of-
        // death guard lives inside `next_fixed_step`.
        let fixed_dt = self.time.fixed_delta_secs();
        while self.time.next_fixed_step() {
            self.fixed_update(fixed_dt);
        }

        // Toggle the debug path overlay with `P`.
        if self.input.is_key_just_pressed(winit::keyboard::KeyCode::KeyP) {
            self.show_path_debug = !self.show_path_debug;
            log::info!("Path debug overlay: {}", self.show_path_debug);
        }

        // Per-frame (variable-rate) work: view matrix, hover pick, batching.
        self.prepare_frame();

        // Reset edge-triggered input now that this frame's logic has read it.
        self.input.clear_frame_state();
    }

    /// Fixed-timestep logic update. `dt` is always [`Time::fixed_delta_secs`],
    /// so simulation behaves identically regardless of frame rate. Future game
    /// logic (enemies, projectiles, towers, economy) steps here.
    fn fixed_update(&mut self, dt: f32) {
        if !self.player_stats.is_alive() {
            return;
        }
        self.time_elapsed += dt;
        self.camera_controller.update_camera(&mut self.camera, dt);
        
        // Spawn a new enemy every 2 seconds for testing
        if (self.time_elapsed % 2.0) < dt {
            if let Some(start_pos) = self.map.path.start() {
                self.enemy_manager.spawn_enemy(Box::new(crate::game::enemies::basic_enemy::BasicEnemy::new(start_pos)));
            }
        }
        
        let (killed, escaped) = self.enemy_manager.update(dt, &self.map.path);

        if escaped > 0 {
            self.player_stats.take_damage(escaped as i32);
            if !self.player_stats.is_alive() {
                self.placement_status = "GAME OVER!".to_string();
            }
            self.update_window_title();
        }

        for (pos, reward) in killed {
            self.economy.add_money(reward as i32);
            self.update_window_title();
            
            // Calculate 3D position for the popup
            let world_x = (pos.x - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
            let world_z = (pos.y - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
            
            let gx = pos.x.round() as i32;
            let gy = pos.y.round() as i32;
            let tile_y = self.map.get_tile(gx, gy).map(|t| t.grid_y).unwrap_or(0);
            let world_y = tile_y as f32 * TILE_WORLD_SIZE + TILE_WORLD_SIZE * 0.5 + 0.05;

            self.popups.push(GoldPopup {
                position: glam::Vec3::new(world_x, world_y + 0.02, world_z),
                lifetime: 1.0,
                amount: reward,
            });
        }
        
        // Update towers and handle projectiles
        let new_projectiles = self.tower_manager.update_all(dt, self.enemy_manager.get_enemies());
        self.projectiles.extend(new_projectiles);

        // Update active projectiles
        let mut to_remove = Vec::new();
        for (i, proj) in self.projectiles.iter_mut().enumerate() {
            if proj.update(dt) {
                to_remove.push(i);
                
                // Damage enemies near the target position
                for enemy in &mut self.enemy_manager.enemies {
                    let dist = enemy.get_position().distance(proj.target_position);
                    if dist < 0.5 { // 0.5 tile hit radius
                        enemy.take_damage(proj.damage);
                    }
                }
            }
        }

        // Remove dead projectiles
        for i in to_remove.into_iter().rev() {
            self.projectiles.swap_remove(i);
        }

        // Update gold popups
        for popup in &mut self.popups {
            popup.position.y += 0.04 * dt; // Float upward
            popup.lifetime -= dt;
        }
        self.popups.retain(|p| p.lifetime > 0.0);
    }

    /// Per-frame render preparation: upload the camera matrix, resolve the
    /// hovered tile, and rebuild the dynamic draw batch. Runs once per rendered
    /// frame (variable rate), independent of the fixed logic steps above.
    fn prepare_frame(&mut self) {
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

                let is_valid = self.tower_manager.can_place_tower(
                    &self.map,
                    glam::Vec2::new(gx as f32, gz as f32),
                    crate::game::towers::manager::TowerType::Basic,
                    &self.economy,
                ).is_ok();
                let highlight_color = if is_valid {
                    [0.0, 1.0, 0.0, 0.45] // Green for valid
                } else {
                    [1.0, 0.0, 0.0, 0.45] // Red for invalid
                };

                self.batcher.add_highlight_box(
                    glam::Vec3::new(pos_x, center_y, pos_z),
                    size,
                    highlight_color,
                    self.highlight_texture_id,
                );
            }
        }

        // Draw towers
        for tower in &self.tower_manager.towers {
            let pos = tower.get_position();
            let world_x = (pos.x - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
            let world_z = (pos.y - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
            
            let gx = pos.x.round() as i32;
            let gy = pos.y.round() as i32;
            let tile_y = self.map.get_tile(gx, gy).map(|t| t.grid_y).unwrap_or(0);
            
            let world_y = tile_y as f32 * TILE_WORLD_SIZE + TILE_WORLD_SIZE * 0.5;

            // Draw tower model
            // The model is around 1 unit high, we want it to be roughly a tile size
            let scale = TILE_WORLD_SIZE * 0.6;
            
            // BasicTower's rotation is an angle in the XY plane where +X is 0, +Y is PI/2.
            // In 3D, our camera XZ plane maps from 2D XY. So +Y in 2D is +Z in 3D.
            // A rotation of angle around Y axis from +X to +Z is exactly the same as 2D angle (if Y is UP).
            // Actually, in 3D Y is up, so rotation around Y from +X to +Z is a negative angle.
            let rot = tower.get_rotation();
            let rot_y = -rot;

            let vertices = self.tower_model.generate_vertices(
                glam::Vec3::new(world_x, world_y, world_z),
                scale,
                rot_y,
                [1.0, 1.0, 1.0, 1.0], // White tint -> show the model's baked colors
            );
            self.batcher.add_model(vertices, self.highlight_texture_id);
        }

        // Draw enemies
        for enemy in self.enemy_manager.get_enemies() {
            let pos = enemy.get_position();
            let health = enemy.get_health();
            
            // Convert grid pos to world pos
            let world_x = (pos.x - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
            let world_z = (pos.y - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
            
            // Determine Y based on terrain at that grid pos (or just slightly above path level)
            // The path is at grid_y = 0 usually, but let's query the map.
            // Since path positions are continuous, we sample the closest integer tile.
            let gx = pos.x.round() as i32;
            let gy = pos.y.round() as i32;
            let tile_y = self.map.get_tile(gx, gy).map(|t| t.grid_y).unwrap_or(0);
            
            let world_y = tile_y as f32 * TILE_WORLD_SIZE + TILE_WORLD_SIZE * 0.5 + 0.05;

            // Draw enemy model
            let vertices = self.enemy_model.generate_vertices(
                glam::Vec3::new(world_x, world_y, world_z),
                0.05, // scale
                self.time_elapsed * 2.0, // rotation for visual effect
                [1.0, 0.2, 0.2, 1.0], // color
            );
            self.batcher.add_model(vertices, self.highlight_texture_id);

            // Draw health bar
            let max_health = 100.0; // Hardcoded for now based on BasicEnemy
            let health_pct = (health / max_health).clamp(0.0, 1.0);
            
            let bar_width = 0.08;
            let bar_height = 0.01;
            let bar_y = world_y + 0.06; // Above the enemy
            
            // Red background (missing health)
            self.batcher.add_sprite(
                crate::renderer::sprite::Sprite::new_colored(
                    glam::Vec3::new(world_x, bar_y, world_z),
                    glam::Vec2::new(bar_width, bar_height),
                    self.highlight_texture_id,
                    0.0,
                    [1.0, 0.0, 0.0, 1.0], // Red
                ),
                crate::renderer::sprite::SpriteAlignment::Billboard,
            );
            
            // Green foreground (current health)
            let current_bar_width = bar_width * health_pct;
            let offset_x = (bar_width - current_bar_width) / 2.0; // Left-align the green bar
            self.batcher.add_sprite(
                crate::renderer::sprite::Sprite::new_colored(
                    glam::Vec3::new(world_x - offset_x, bar_y + 0.001, world_z),
                    glam::Vec2::new(current_bar_width, bar_height),
                    self.highlight_texture_id,
                    0.0,
                    [0.0, 1.0, 0.0, 1.0], // Green
                ),
                crate::renderer::sprite::SpriteAlignment::Billboard,
            );
        }

        // Draw projectiles
        for proj in &self.projectiles {
            let pos = proj.position;
            let world_x = (pos.x - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
            let world_z = (pos.y - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
            
            // Interpolate the exact 3D world height from start to target
            let start_gx = proj.start_position.x.round() as i32;
            let start_gy = proj.start_position.y.round() as i32;
            let start_tile_y = self.map.get_tile(start_gx, start_gy).map(|t| t.grid_y).unwrap_or(0);
            
            let target_gx = proj.target_position.x.round() as i32;
            let target_gy = proj.target_position.y.round() as i32;
            let target_tile_y = self.map.get_tile(target_gx, target_gy).map(|t| t.grid_y).unwrap_or(0);

            // A tile's top surface (where towers/enemies sit) is half a cube
            // above its grid_y centre, so every height is measured from
            // `(tile_y + 0.5) * TILE`. `spawn_height_offset` is the barrel
            // height above that surface, in tile units.
            let start_world_y =
                (start_tile_y as f32 + 0.5 + proj.spawn_height_offset) * TILE_WORLD_SIZE;
            // Aim slightly above the ground to hit the enemy body (not under it).
            let target_world_y = (target_tile_y as f32 + 0.5) * TILE_WORLD_SIZE + 0.2 * TILE_WORLD_SIZE;
            
            let total_dist = proj.start_position.distance(proj.target_position);
            let current_dist = proj.position.distance(proj.target_position);
            let t = if total_dist > 0.0 { 1.0 - (current_dist / total_dist) } else { 1.0 };
            
            let world_y = start_world_y * (1.0 - t) + target_world_y * t;
            
            // The generated sphere has radius 1.0, so let's scale it to 0.008
            let scale = 0.008; 
            let vertices = self.projectile_model.generate_vertices(
                glam::Vec3::new(world_x, world_y, world_z),
                scale,
                0.0,
                [1.0, 1.0, 0.0, 1.0], // Yellow
            );
            self.batcher.add_model(vertices, self.highlight_texture_id);
        }

        // Draw popups
        for popup in &self.popups {
            let size = glam::Vec2::new(0.06, 0.03); // Billboard size for the +10 popup
            let alpha = popup.lifetime.clamp(0.0, 1.0);
            self.batcher.add_sprite(
                crate::renderer::sprite::Sprite::new_colored(
                    popup.position,
                    size,
                    self.popup_texture_id,
                    0.0,
                    [1.0, 1.0, 1.0, alpha], // Fade out as lifetime decays
                ),
                crate::renderer::sprite::SpriteAlignment::Billboard,
            );
        }

        // Debug: draw the enemy path as ground markers when enabled.
        if self.show_path_debug {
            self.draw_path_debug();
        }

        // Compile batches and upload to the GPU.
        self.batcher.finalize(&self.device, &self.queue);
    }

    /// Queues translucent ground markers tracing the map's enemy path into the
    /// dynamic batch. Markers are sampled along each segment so any path shape
    /// (including diagonals) is visualised.
    fn draw_path_debug(&mut self) {
        // Snapshot the waypoints so the immutable borrow of `self.map` is
        // released before mutably borrowing `self.batcher` below.
        let waypoints: Vec<glam::Vec2> = self.map.path.waypoints().to_vec();
        if waypoints.len() < 2 {
            return;
        }

        let mw = self.map.width as f32;
        let mh = self.map.height as f32;
        let tex = self.highlight_texture_id;
        // Float just above the (flat) path tiles' top surface.
        let y = TILE_WORLD_SIZE * 0.5 + 0.012;
        let size = glam::Vec3::splat(TILE_WORLD_SIZE * 0.3);

        let to_world = |w: glam::Vec2| {
            (
                (w.x - mw / 2.0) * TILE_WORLD_SIZE,
                (w.y - mh / 2.0) * TILE_WORLD_SIZE,
            )
        };

        for pair in waypoints.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let steps = (a.distance(b) / 0.15).ceil().max(1.0) as i32;
            for i in 0..=steps {
                let p = a.lerp(b, i as f32 / steps as f32);
                let (wx, wz) = to_world(p);
                self.batcher.add_overlay_quad(
                    glam::Vec3::new(wx, y, wz),
                    size,
                    PATH_DEBUG_COLOR,
                    tex,
                );
            }
        }
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
                // Track in the central input state, then keep driving the
                // camera controller as before.
                self.input.process_key(*code, *key_state);
                self.camera_controller.process_key(*code, key_state.is_pressed())
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                self.input.process_mouse_button(*button, *state);

                if *button == winit::event::MouseButton::Left && *state == winit::event::ElementState::Pressed {
                    if let Some((gx, gz)) = self.hovered_tile {
                        let pos = glam::Vec2::new(gx as f32, gz as f32);
                        let tower_type = crate::game::towers::manager::TowerType::Basic;
                        match self.tower_manager.place_tower(&self.map, pos, tower_type, &mut self.economy) {
                            Ok(_) => {
                                log::info!("Placed tower at {}, {}", gx, gz);
                                self.placement_status = format!("Placed basic tower at ({}, {})", gx, gz);
                                self.update_window_title();
                            }
                            Err(e) => {
                                log::warn!("Failed to place tower: {}", e);
                                self.placement_status = format!("Failed to place tower: {}", e);
                                self.update_window_title();
                            }
                        }
                    }
                }

                false
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .set_mouse_position(position.x as f32, position.y as f32);
                // Returns false so `app.rs` still updates the hover cursor.
                false
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            _ => false,
        }
    }

    /// Read-only access to the current keyboard/mouse state for game logic.
    pub fn input_state(&self) -> &crate::engine::input::InputState {
        &self.input
    }
}

/// World-space edge length of a single terrain tile/cube. Terrain meshing and
/// cursor picking both use this so they stay in sync.
pub const TILE_WORLD_SIZE: f32 = 0.12;

/// Lowest cube level drawn for the terrain skirt (border depth). Meshing and
/// picking share this so a tile's pickable column matches its visible extent.
const TERRAIN_BASE_LEVEL: i32 = -2;


/// How much larger the highlight shell is than the block it wraps, so it sits
/// just outside the terrain faces and avoids z-fighting.
const HIGHLIGHT_INFLATE: f32 = 0.006;

/// Translucent red tint for the debug enemy-path markers.
const PATH_DEBUG_COLOR: [f32; 4] = [1.0, 0.25, 0.15, 0.9];

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

fn create_gold_texture(text: &str) -> image::RgbaImage {
    let width = 32;
    let height = 16;
    let mut img = image::RgbaImage::new(width, height);
    
    // Tiny 3x5 font: 0-9 and '+'
    let get_char_bitmap = |c: char| -> &'static [u8; 5] {
        match c {
            '+' => &[2, 2, 7, 2, 2],
            '0' => &[7, 5, 5, 5, 7],
            '1' => &[2, 6, 2, 2, 7],
            '2' => &[7, 1, 7, 4, 7],
            '3' => &[7, 1, 7, 1, 7],
            '4' => &[5, 5, 7, 1, 1],
            '5' => &[7, 4, 7, 1, 7],
            '6' => &[7, 4, 7, 5, 7],
            '7' => &[7, 1, 2, 2, 2],
            '8' => &[7, 5, 7, 5, 7],
            '9' => &[7, 5, 7, 1, 7],
            _ => &[0, 0, 0, 0, 0],
        }
    };
    
    // Calculate total width to center the text
    let char_width = 3;
    let spacing = 1;
    let total_width = text.len() as i32 * char_width + (text.len() as i32 - 1) * spacing;
    let start_x = (width as i32 - total_width) / 2;
    let start_y = (height as i32 - 5) / 2;
    
    for (char_idx, c) in text.chars().enumerate() {
        let bitmap = get_char_bitmap(c);
        let cx = start_x + char_idx as i32 * (char_width + spacing);
        
        for row in 0..5 {
            let val = bitmap[row];
            let cy = start_y + row as i32;
            for col in 0..3 {
                // Check if bit (2 - col) is set
                let bit = 1 << (2 - col);
                if (val & bit) != 0 {
                    let px = cx + col;
                    if px >= 0 && px < width as i32 && cy >= 0 && cy < height as i32 {
                        img.put_pixel(px as u32, cy as u32, image::Rgba([255, 220, 0, 255]));
                    }
                }
            }
        }
    }
    
    // Outline pass: for any yellow pixel, check its 8 neighbors. If they are empty, make them black.
    let mut yellow_pixels = std::collections::HashSet::new();
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] > 0 && p[0] == 255 && p[1] == 220 {
                yellow_pixels.insert((x as i32, y as i32));
            }
        }
    }
    
    for &(x, y) in &yellow_pixels {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    if !yellow_pixels.contains(&(nx, ny)) {
                        img.put_pixel(nx as u32, ny as u32, image::Rgba([0, 0, 0, 255]));
                    }
                }
            }
        }
    }
    
    img
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
