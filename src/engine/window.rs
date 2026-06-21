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
    /// Per-frame batcher for UI overlays (rendered on top of everything without depth test).
    ui_batcher: crate::renderer::batch::SpriteBatcher,
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
    /// Wave manager to coordinate enemy spawning.
    pub wave_manager: crate::game::wave_manager::WaveManager,
    /// Gold reward popup sprites.
    popups: Vec<GoldPopup>,
    /// Texture id for the gold popup sprite.
    popup_texture_id: usize,
    /// Texture for the wave UI display.
    wave_ui_texture: crate::renderer::texture::Texture,
    /// Texture id for the wave UI display.
    wave_ui_texture_id: usize,
    /// Text of the last wave UI texture upload.
    last_wave_ui_text: String,
    /// The current game state.
    pub game_state: crate::game::game_state::GameState,
    /// Whether the player has continued playing in sandbox mode after victory.
    pub continued_after_victory: bool,
    /// egui context for managing UI state.
    egui_ctx: egui::Context,
    /// egui winit event handler state.
    egui_state: egui_winit::State,
    /// egui wgpu renderer.
    egui_renderer: egui_wgpu::Renderer,
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
        let ui_batcher = crate::renderer::batch::SpriteBatcher::new();

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

        // Generate and register wave UI texture
        let last_wave_ui_text = "MAIN MENU".to_string();
        let wave_ui_image = create_multiline_text_texture("ISOGUARD", "PRESS ENTER TO START", 160, 48, [255, 220, 0, 255], [0, 255, 255, 255]);
        let wave_ui_texture = crate::renderer::texture::Texture::from_image(
            &device,
            &queue,
            &image::DynamicImage::ImageRgba8(wave_ui_image),
            Some("wave_ui"),
        )
        .expect("wave UI texture should be valid");
        let wave_ui_texture_id = renderer.register_texture(&device, &wave_ui_texture);

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

        let enemy_manager = crate::game::enemies::EnemyManager::new();

        let waves = crate::game::wave_manager::WaveManager::get_default_waves();
        let wave_manager = crate::game::wave_manager::WaveManager::new(waves);

        let egui_ctx = egui::Context::default();
        let viewport_id = egui_ctx.viewport_id();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            viewport_id,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                predictable_texture_filtering: false,
            },
        );

        let state = Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: true,
            renderer,
            batcher,
            ui_batcher,
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
            wave_manager,
            popups: Vec::new(),
            popup_texture_id,
            wave_ui_texture,
            wave_ui_texture_id,
            last_wave_ui_text,
            game_state: crate::game::game_state::GameState::MainMenu,
            continued_after_victory: false,
            egui_ctx,
            egui_state,
            egui_renderer,
        };

        state.update_window_title();

        Ok(state)
    }

    pub fn start_game(&mut self) {
        self.player_stats = crate::game::player_stats::PlayerStats::new(5);
        self.economy = crate::game::economy::Economy::new(200);

        let waves = crate::game::wave_manager::WaveManager::get_default_waves();
        self.wave_manager = crate::game::wave_manager::WaveManager::new(waves);
        self.wave_manager.start_wave(0);

        self.enemy_manager = crate::game::enemies::EnemyManager::new();
        self.tower_manager = crate::game::towers::manager::TowerManager::new();
        self.projectiles.clear();
        self.popups.clear();

        self.continued_after_victory = false;
        self.game_state = crate::game::game_state::GameState::Playing;
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
        let wave_msg = self.wave_manager.get_status_message();
        let title = format!(
            "Isoguard - Lives: {}/{} | Gold: {} | Basic Tower Cost: {} | Status: {} | {}",
            self.player_stats.lives,
            self.player_stats.max_lives,
            self.economy.money,
            crate::game::towers::manager::TowerType::Basic.cost(),
            self.placement_status,
            wave_msg
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

        // Handle keyboard transitions
        if self.input.is_key_just_pressed(winit::keyboard::KeyCode::Escape) {
            self.game_state = self.game_state.transition_on_escape();
            log::info!("Toggled pause state to: {:?}", self.game_state);
        }
        if self.input.is_key_just_pressed(winit::keyboard::KeyCode::Enter)
            || self.input.is_key_just_pressed(winit::keyboard::KeyCode::Space)
        {
            let old_state = self.game_state;
            self.game_state = self.game_state.transition_on_start();
            if old_state != self.game_state {
                let is_continue = old_state == crate::game::game_state::GameState::Victory
                    && self.input.is_key_just_pressed(winit::keyboard::KeyCode::Space);

                if is_continue {
                    self.continued_after_victory = true;
                    log::info!("Continuing game in sandbox mode! State: {:?}", self.game_state);
                } else {
                    self.start_game();
                    log::info!("Started game! New state: {:?}", self.game_state);
                }
            }
        }

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
        // Camera movement should work even in menus/paused states so the player can navigate.
        self.camera_controller.update_camera(&mut self.camera, dt);

        if self.game_state != crate::game::game_state::GameState::Playing {
            return;
        }

        if !self.player_stats.is_alive() {
            self.game_state = crate::game::game_state::GameState::GameOver;
            self.placement_status = "GAME OVER!".to_string();
            self.update_window_title();
            return;
        }

        if self.wave_manager.state == crate::game::wave_manager::WaveState::CompletedAll && !self.continued_after_victory {
            self.game_state = crate::game::game_state::GameState::Victory;
            self.placement_status = "VICTORY!".to_string();
            self.update_window_title();
            return;
        }

        self.time_elapsed += dt;
        
        let active_enemy_count = self.enemy_manager.get_enemies().len();
        let prev_state = self.wave_manager.state;
        let prev_wave_num = self.wave_manager.current_wave_number();
        let prev_timer = self.wave_manager.inter_wave_timer;

        if let Some(enemy_type) = self.wave_manager.update(dt, active_enemy_count) {
            match enemy_type {
                crate::game::wave_manager::EnemyType::Basic => {
                    if let Some(start_pos) = self.map.path.start() {
                        let enemy = crate::game::enemies::basic_enemy::BasicEnemy::new(start_pos);
                        self.enemy_manager.spawn_enemy(Box::new(enemy));
                    }
                }
            }
        }

        let state_changed = prev_state != self.wave_manager.state;
        let wave_num_changed = prev_wave_num != self.wave_manager.current_wave_number();
        let timer_changed = (prev_timer * 10.0).round() != (self.wave_manager.inter_wave_timer * 10.0).round();
        if state_changed || wave_num_changed || timer_changed {
            self.update_window_title();
        }
        
        let (killed, escaped) = self.enemy_manager.update(dt, &self.map.path);

        if escaped > 0 {
            self.player_stats.take_damage(escaped as i32);
            if !self.player_stats.is_alive() {
                self.game_state = crate::game::game_state::GameState::GameOver;
                self.placement_status = "GAME OVER!".to_string();
                self.update_window_title();
                return;
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
            popup.position.y += 0.08 * dt; // Float upward
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
        self.ui_batcher.clear();

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
            let size = glam::Vec2::new(0.12, 0.06); // Billboard size for the +10 popup
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

        // --- Wave/Game Status UI ---
        let (line1, line2, color1, color2) = match self.game_state {
            crate::game::game_state::GameState::MainMenu => (
                "ISOGUARD".to_string(),
                "PRESS ENTER TO START".to_string(),
                [255, 220, 0, 255],     // Yellow/Gold
                [0, 255, 255, 255],     // Cyan
            ),
            crate::game::game_state::GameState::Paused => (
                "GAME PAUSED".to_string(),
                "PRESS ESC TO RESUME".to_string(),
                [255, 120, 0, 255],     // Orange
                [255, 255, 255, 255],   // White
            ),
            crate::game::game_state::GameState::GameOver => {
                let wave_reached = self.wave_manager.current_wave_number();
                (
                    "GAME OVER!".to_string(),
                    format!("WAVE {} REACHED - PRESS ENTER", wave_reached),
                    [255, 50, 50, 255],     // Red
                    [255, 255, 255, 255],   // White
                )
            }
            crate::game::game_state::GameState::Victory => {
                let stats_line = format!("LIVES:{} GOLD:{}", self.player_stats.lives, self.economy.money);
                (
                    format!("VICTORY!  {}", stats_line),
                    "ENTER:RESTART  SPACE:CONTINUE".to_string(),
                    [0, 255, 255, 255],     // Cyan
                    [255, 220, 0, 255],     // Yellow/Gold
                )
            }
            crate::game::game_state::GameState::Playing => {
                let stats_line = format!("LIVES: {}  GOLD: {}", self.player_stats.lives, self.economy.money);
                match self.wave_manager.state {
                    crate::game::wave_manager::WaveState::NotStarted => (
                        "WAITING TO START".to_string(),
                        stats_line,
                        [255, 220, 0, 255],
                        [255, 255, 255, 255],
                    ),
                    crate::game::wave_manager::WaveState::Spawning | crate::game::wave_manager::WaveState::WaitingForClean => (
                        format!("WAVE {}", self.wave_manager.current_wave_number()),
                        stats_line,
                        [255, 220, 0, 255],
                        [255, 255, 255, 255],
                    ),
                    crate::game::wave_manager::WaveState::InterWaveDelay => {
                        if self.wave_manager.inter_wave_timer > 7.0 {
                            (
                                format!("WAVE {} COMPLETE!", self.wave_manager.current_wave_number()),
                                stats_line,
                                [0, 255, 100, 255],
                                [255, 255, 255, 255],
                            )
                        } else {
                            let next_wave = self.wave_manager.current_wave_number() + 1;
                            (
                                format!("WAVE {} INCOMING: {:.1}S", next_wave, self.wave_manager.inter_wave_timer),
                                stats_line,
                                [255, 120, 0, 255],
                                [255, 255, 255, 255],
                            )
                        }
                    }
                    crate::game::wave_manager::WaveState::CompletedAll => (
                        "VICTORY!".to_string(),
                        stats_line,
                        [0, 255, 255, 255],
                        [255, 255, 255, 255],
                    ),
                }
            }
        };

        let cache_key = format!("{}_{}_{:?}_{:?}", line1, line2, color1, color2);
        if cache_key != self.last_wave_ui_text {
            let img = create_multiline_text_texture(&line1, &line2, 160, 48, color1, color2);
            let size = wgpu::Extent3d {
                width: 160,
                height: 48,
                depth_or_array_layers: 1,
            };
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &self.wave_ui_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 160),
                    rows_per_image: Some(48),
                },
                size,
            );
            self.last_wave_ui_text = cache_key;
        }

        // Draw Wave UI Billboard
        if self.game_state != crate::game::game_state::GameState::Playing {
            let ortho_height = 2.0 / self.camera.zoom;
            let pixel_scale = ortho_height / self.config.height as f32;

            let base_w = 640.0;
            let base_h = 192.0;

            // Clamp the UI billboard size if the screen is too small, so it never gets cut off
            let mut scale_factor = 1.0f32;
            let margin_pct = 0.90f32;
            let screen_w = self.config.width as f32;
            let screen_h = self.config.height as f32;
            if screen_w * margin_pct < base_w {
                scale_factor = scale_factor.min((screen_w * margin_pct) / base_w);
            }
            if screen_h * margin_pct < base_h {
                scale_factor = scale_factor.min((screen_h * margin_pct) / base_h);
            }

            let scaled_w = base_w * scale_factor;
            let scaled_h = base_h * scale_factor;
            let sprite_size = glam::Vec2::new(scaled_w * pixel_scale, scaled_h * pixel_scale);

            let pos = self.camera.target;

            self.ui_batcher.add_sprite(
                crate::renderer::sprite::Sprite::new_colored(
                    pos,
                    sprite_size,
                    self.wave_ui_texture_id,
                    0.0,
                    [1.0, 1.0, 1.0, 1.0], // Tint
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
        self.ui_batcher.finalize(&self.device, &self.queue);
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

        // Draw UI overlays on top of the 3D scene with depth testing disabled.
        self.renderer.render_ui_batches(
            &view,
            &self.device,
            &self.queue,
            &[&self.ui_batcher],
        )?;

        // 1. Get raw input from winit and begin frame
        let raw_input = self.egui_state.take_egui_input(&self.window);
        self.egui_ctx.begin_pass(raw_input);

        // 2. Draw HUD if in GameState::Playing
        if self.game_state == crate::game::game_state::GameState::Playing {
            // Draw lives panel (top-left)
            egui::Area::new(egui::Id::new("hud_lives"))
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(20.0, 20.0))
                .show(&self.egui_ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(180))
                        .corner_radius(8.0)
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 75, 75)))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("❤️").size(24.0));
                                ui.label(egui::RichText::new(format!("{}", self.player_stats.lives))
                                    .font(egui::FontId::proportional(20.0))
                                    .color(egui::Color32::WHITE)
                                    .strong());
                            });
                        });
                });

            // Draw gold panel (top-right)
            egui::Area::new(egui::Id::new("hud_gold"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 20.0))
                .show(&self.egui_ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(180))
                        .corner_radius(8.0)
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 215, 0)))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("💰").size(24.0));
                                ui.label(egui::RichText::new(format!("{}", self.economy.money))
                                    .font(egui::FontId::proportional(20.0))
                                    .color(egui::Color32::WHITE)
                                    .strong());
                            });
                        });
                });

            // Draw wave panel (top-center)
            egui::Area::new(egui::Id::new("hud_wave"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 20.0))
                .show(&self.egui_ctx, |ui| {
                    let wave_text = match self.wave_manager.state {
                        crate::game::wave_manager::WaveState::NotStarted => "Waiting to Start".to_string(),
                        crate::game::wave_manager::WaveState::Spawning | crate::game::wave_manager::WaveState::WaitingForClean => {
                            format!("Wave {}/{}", self.wave_manager.current_wave_number(), self.wave_manager.waves.len())
                        }
                        crate::game::wave_manager::WaveState::InterWaveDelay => {
                            if self.wave_manager.inter_wave_timer > 7.0 {
                                format!("Wave {} Complete!", self.wave_manager.current_wave_number())
                            } else {
                                let next_wave = self.wave_manager.current_wave_number() + 1;
                                format!("Wave {} Incoming: {:.1}s", next_wave, self.wave_manager.inter_wave_timer)
                            }
                        }
                        crate::game::wave_manager::WaveState::CompletedAll => "Victory!".to_string(),
                    };
                    let border_color = match self.wave_manager.state {
                        crate::game::wave_manager::WaveState::NotStarted => egui::Color32::from_rgb(0, 180, 255),
                        crate::game::wave_manager::WaveState::Spawning | crate::game::wave_manager::WaveState::WaitingForClean => {
                            egui::Color32::from_rgb(0, 255, 100)
                        }
                        crate::game::wave_manager::WaveState::InterWaveDelay => {
                            if self.wave_manager.inter_wave_timer > 7.0 {
                                egui::Color32::from_rgb(0, 255, 100)
                            } else {
                                egui::Color32::from_rgb(255, 120, 0)
                            }
                        }
                        crate::game::wave_manager::WaveState::CompletedAll => egui::Color32::from_rgb(0, 255, 255),
                    };

                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(180))
                        .corner_radius(8.0)
                        .stroke(egui::Stroke::new(1.5, border_color))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚔️").size(24.0));
                                ui.label(egui::RichText::new(wave_text)
                                    .font(egui::FontId::proportional(20.0))
                                    .color(egui::Color32::WHITE)
                                    .strong());
                            });
                        });
                });
        }

        // 3. End frame and tessellate
        let full_output = self.egui_ctx.end_pass();
        
        // Handle egui platform output (textures and viewports)
        self.egui_state.handle_platform_output(&self.window, full_output.platform_output);

        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        // 4. Update egui textures
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        // 5. Create command encoder for egui buffer updates and rendering
        let mut egui_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Egui Encoder"),
        });

        // 6. Update egui buffers on GPU
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut egui_encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        // 7. Render egui using a render pass with LoadOp::Load
        {
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            }).forget_lifetime();

            self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        // 8. Submit egui commands & free textures
        self.queue.submit(std::iter::once(egui_encoder.finish()));

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        output.present();

        Ok(())
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        let response = self.egui_state.on_window_event(&self.window, event);
        if response.consumed {
            return true;
        }
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

                if self.game_state == crate::game::game_state::GameState::Playing
                    && *button == winit::event::MouseButton::Left
                    && *state == winit::event::ElementState::Pressed
                {
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

fn create_multiline_text_texture(
    line1: &str,
    line2: &str,
    width: u32,
    height: u32,
    color1: [u8; 4],
    color2: [u8; 4],
) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(width, height);
    
    // Tiny 3x5 font
    let get_char_bitmap = |c: char| -> &'static [u8; 5] {
        match c {
            '0' => &[7, 5, 5, 5, 7],
            '1' => &[2, 2, 2, 2, 2],
            '2' => &[7, 1, 7, 4, 7],
            '3' => &[7, 1, 7, 1, 7],
            '4' => &[5, 5, 7, 1, 1],
            '5' => &[7, 4, 7, 1, 7],
            '6' => &[7, 4, 7, 5, 7],
            '7' => &[7, 1, 1, 1, 1],
            '8' => &[7, 5, 7, 5, 7],
            '9' => &[7, 5, 7, 1, 7],
            'A' | 'a' => &[2, 5, 7, 5, 5],
            'B' | 'b' => &[6, 5, 6, 5, 6],
            'C' | 'c' => &[7, 4, 4, 4, 7],
            'D' | 'd' => &[6, 5, 5, 5, 6],
            'E' | 'e' => &[7, 4, 6, 4, 7],
            'F' | 'f' => &[7, 4, 6, 4, 4],
            'G' | 'g' => &[7, 4, 5, 5, 7],
            'H' | 'h' => &[5, 5, 7, 5, 5],
            'I' | 'i' => &[7, 2, 2, 2, 7],
            'J' | 'j' => &[1, 1, 1, 5, 2],
            'K' | 'k' => &[5, 5, 6, 5, 5],
            'L' | 'l' => &[4, 4, 4, 4, 7],
            'M' | 'm' => &[5, 7, 5, 5, 5],
            'N' | 'n' => &[5, 7, 7, 5, 5],
            'O' | 'o' => &[7, 5, 5, 5, 7],
            'P' | 'p' => &[7, 5, 7, 4, 4],
            'Q' | 'q' => &[7, 5, 5, 7, 1],
            'R' | 'r' => &[6, 5, 6, 5, 5],
            'S' | 's' => &[7, 4, 7, 1, 7],
            'T' | 't' => &[7, 2, 2, 2, 2],
            'U' | 'u' => &[5, 5, 5, 5, 7],
            'V' | 'v' => &[5, 5, 5, 5, 2],
            'W' | 'w' => &[5, 5, 5, 7, 5],
            'X' | 'x' => &[5, 5, 2, 5, 5],
            'Y' | 'y' => &[5, 5, 2, 2, 2],
            'Z' | 'z' => &[7, 1, 2, 4, 7],
            '.' => &[0, 0, 0, 0, 2],
            ':' => &[0, 2, 0, 2, 0],
            '!' => &[2, 2, 2, 0, 2],
            '+' => &[2, 2, 7, 2, 2],
            '-' => &[0, 0, 7, 0, 0],
            ' ' => &[0, 0, 0, 0, 0],
            _ => &[0, 0, 0, 0, 0],
        }
    };

    // Draw line 1 (vertical center of top half, e.g. height / 4 = 12 for height = 48)
    if !line1.is_empty() {
        let text_width = line1.len() as i32 * 4 - 1;
        let start_x = (width as i32 - text_width) / 2;
        let start_y = (height as i32 / 4) - 2;
        
        for (char_idx, c) in line1.chars().enumerate() {
            let bitmap = get_char_bitmap(c);
            let cx = start_x + char_idx as i32 * 4;
            
            for row in 0..5 {
                let row_val = bitmap[row];
                let cy = start_y + row as i32;
                for col in 0..3 {
                    let bit = (row_val >> (2 - col)) & 1;
                    if bit == 1 {
                        let px = cx + col;
                        if px >= 0 && px < width as i32 && cy >= 0 && cy < height as i32 {
                            img.put_pixel(px as u32, cy as u32, image::Rgba(color1));
                        }
                    }
                }
            }
        }
    }

    // Draw line 2 (vertical center of bottom half, e.g. 3 * height / 4 = 36 for height = 48)
    if !line2.is_empty() {
        let text_width = line2.len() as i32 * 4 - 1;
        let start_x = (width as i32 - text_width) / 2;
        let start_y = (3 * height as i32 / 4) - 2;
        
        for (char_idx, c) in line2.chars().enumerate() {
            let bitmap = get_char_bitmap(c);
            let cx = start_x + char_idx as i32 * 4;
            
            for row in 0..5 {
                let row_val = bitmap[row];
                let cy = start_y + row as i32;
                for col in 0..3 {
                    let bit = (row_val >> (2 - col)) & 1;
                    if bit == 1 {
                        let px = cx + col;
                        if px >= 0 && px < width as i32 && cy >= 0 && cy < height as i32 {
                            img.put_pixel(px as u32, cy as u32, image::Rgba(color2));
                        }
                    }
                }
            }
        }
    }

    // Outline pass: for any pixel of our text colors, check its 8 neighbors. If they are empty, make them black.
    let mut text_pixels = std::collections::HashSet::new();
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            let is_colored1 = p[3] > 0 && p[0] == color1[0] && p[1] == color1[1] && p[2] == color1[2];
            let is_colored2 = p[3] > 0 && p[0] == color2[0] && p[1] == color2[1] && p[2] == color2[2];
            if is_colored1 || is_colored2 {
                text_pixels.insert((x as i32, y as i32));
            }
        }
    }
    
    for &(x, y) in &text_pixels {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    if !text_pixels.contains(&(nx, ny)) {
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

#[allow(dead_code)]
fn dummy_compile_test(device: &wgpu::Device, window: &winit::window::Window) {
    let egui_ctx = egui::Context::default();
    let viewport_id = egui_ctx.viewport_id();
    let _egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        viewport_id,
        window,
        Some(window.scale_factor() as f32),
        None,
        None,
    );
    let _egui_renderer = egui_wgpu::Renderer::new(
        device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        egui_wgpu::RendererOptions {
            msaa_samples: 1,
            depth_stencil_format: None,
            dithering: true,
            predictable_texture_filtering: false,
        },
    );
}
