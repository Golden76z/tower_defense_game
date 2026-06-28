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
    /// Per-frame batcher for scene overlays (depth tested but no depth writing).
    overlay_batcher: crate::renderer::batch::SpriteBatcher,
    /// Cached static terrain mesh, rebuilt only when the map changes.
    terrain_batcher: crate::renderer::batch::SpriteBatcher,
    map: crate::game::map::Map,
    enemy_manager: crate::game::enemies::EnemyManager,
    /// 3D model for the enemy.
    enemy_model: crate::renderer::model::Model,
    /// Texture id for the enemy sprite.
    _enemy_texture_id: usize,
    /// 3D model for the fast enemy.
    fast_enemy_model: crate::renderer::model::Model,
    /// Texture id for the fast enemy sprite.
    _fast_enemy_texture_id: usize,
    /// Texture id of a 1x1 white texture used to tint overlay quads.
    highlight_texture_id: usize,
    pub tower_manager: crate::game::towers::manager::TowerManager,
    /// 3D model for the tower.
    tower_model: crate::renderer::model::Model,
    /// 3D model for the sniper tower.
    sniper_tower_model: crate::renderer::model::Model,
    /// Currently selected tower type for placement.
    selected_tower_type: crate::game::towers::manager::TowerType,
    /// Active projectiles flying towards enemies.
    projectiles: Vec<crate::game::projectiles::Projectile>,
    /// 3D model for the projectile.
    projectile_model: crate::renderer::model::Model,
    /// Particle system for visual effects.
    particles: crate::game::particles::ParticleSystem,
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
    /// Whether the game has requested to exit.
    pub exit_requested: bool,
    /// egui context for managing UI state.
    egui_ctx: egui::Context,
    /// egui winit event handler state.
    egui_state: egui_winit::State,
    /// egui wgpu renderer.
    egui_renderer: egui_wgpu::Renderer,
    /// Index of the selected tower for upgrades.
    pub selected_tower_index: Option<usize>,
    /// Sound effects system.
    pub audio: crate::engine::audio::AudioSystem,
    /// Whether the options menu is currently displayed on the main menu.
    pub show_options_menu: bool,
    /// Whether the level selection screen is active.
    pub show_level_select: bool,
    /// The index of the currently selected level in the scroll map.
    pub selected_level_point: Option<usize>,
    /// The name of the currently active map.
    pub current_map_name: String,
    /// Counter of spawned enemies for path alternation.
    pub enemy_spawn_count: usize,
    /// Temporary feedback for saving/loading: (message, timer).
    pub save_feedback: Option<(String, f32)>,
    /// Whether the statistics screen is currently displayed on the main menu.
    pub show_stats_screen: bool,
    /// Whether we are currently loading a saved game (to prevent double-incrementing games_played).
    pub is_loading_save: bool,
    /// Cached player statistics loaded from progress file.
    pub statistics: crate::game::progress::GameStatistics,
    spatial_grid: crate::game::spatial_grid::SpatialGrid,
    query_scratch: Vec<usize>,
    pub store_open: bool,
    pub store_animation: f32,
    pub show_ingame_settings: bool,
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
        let overlay_batcher = crate::renderer::batch::SpriteBatcher::new();

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

        // Load the fast enemy texture
        let fast_enemy_image = image::open("assets/sprites/fast_enemy.png")
            .expect("Failed to load fast enemy texture")
            .to_rgba8();
        let fast_enemy_texture = crate::renderer::texture::Texture::from_image(
            &device,
            &queue,
            &image::DynamicImage::ImageRgba8(fast_enemy_image),
            Some("fast_enemy"),
        )
        .expect("fast enemy texture should be valid");
        let fast_enemy_texture_id = renderer.register_texture(&device, &fast_enemy_texture);

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

        // Load 3D model for fast enemies
        let fast_enemy_model = crate::renderer::model::Model::load("assets/models/fast_enemy.obj")
            .expect("Failed to load fast_enemy.obj model");

        // Load 3D model for sniper tower
        let sniper_tower_model = crate::renderer::model::Model::load("assets/models/sniper_tower.obj")
            .expect("Failed to load sniper_tower.obj model");

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

        let audio = crate::engine::audio::AudioSystem::new();

        let progress_save = crate::game::progress::SaveData::load();
        let statistics = progress_save.statistics;

        let spatial_grid = crate::game::spatial_grid::SpatialGrid::new(map.width.max(0) as usize, map.height.max(0) as usize, 2.0);
        let query_scratch = Vec::with_capacity(128);

        let state = Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: true,
            renderer,
            batcher,
            ui_batcher,
            overlay_batcher,
            terrain_batcher,
            map,
            enemy_manager,
            enemy_model,
            _enemy_texture_id: enemy_texture_id,
            fast_enemy_model,
            _fast_enemy_texture_id: fast_enemy_texture_id,
            highlight_texture_id,
            tower_model,
            sniper_tower_model,
            selected_tower_type: crate::game::towers::manager::TowerType::Basic,
            tower_manager: crate::game::towers::manager::TowerManager::new(),
            projectiles: Vec::new(),
            projectile_model,
            particles: crate::game::particles::ParticleSystem::new(),
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
            exit_requested: false,
            egui_ctx,
            egui_state,
            egui_renderer,
            selected_tower_index: None,
            audio,
            show_options_menu: false,
            show_level_select: false,
            selected_level_point: Some(0),
            current_map_name: "procedural".to_string(),
            enemy_spawn_count: 0,
            save_feedback: None,
            show_stats_screen: false,
            is_loading_save: false,
            statistics,
            spatial_grid,
            query_scratch,
            store_open: false,
            store_animation: 0.0,
            show_ingame_settings: false,
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
        self.particles.clear();
        self.selected_tower_index = None;

        self.continued_after_victory = false;
        self.game_state = crate::game::game_state::GameState::Playing;
        self.store_open = false;
        self.store_animation = 0.0;
        self.show_ingame_settings = false;

        if !self.is_loading_save {
            self.statistics.games_played += 1;
            self.save_statistics();
        }
    }

    pub fn load_level(&mut self, name: &str) {
        let map = match name {
            "easy" => crate::game::map::load_map_from_file("assets/maps/easy.json"),
            "medium" => crate::game::map::load_map_from_file("assets/maps/medium.json"),
            "hard" => crate::game::map::load_map_from_file("assets/maps/hard.json"),
            "volcanic" => Ok(crate::game::map::Map::generate_island(20, 20, 9999)),
            "procedural" => Ok(crate::game::map::Map::generate_island(20, 20, 1337)),
            _ => crate::game::map::load_map_from_file("assets/maps/test_map.json"),
        };

        let map = match map {
            Ok(map) => {
                log::info!("Loaded map {} ({}x{})", name, map.width, map.height);
                map
            }
            Err(e) => {
                log::warn!("Failed to load map {}: {e}. Falling back to procedural island.", name);
                crate::game::map::Map::generate_island(20, 20, 1337)
            }
        };

        self.map = map;
        self.spatial_grid = crate::game::spatial_grid::SpatialGrid::new(self.map.width.max(0) as usize, self.map.height.max(0) as usize, 2.0);
        self.current_map_name = name.to_string();
        self.enemy_spawn_count = 0;

        // Reset terrain mesh
        self.rebuild_terrain();

        // Reset game state
        self.start_game();
    }

    pub fn save_game(&mut self) -> Result<(), anyhow::Error> {
        use crate::game::save_game::{SaveData, SavedTower, SavedEnemy};

        let towers = self.tower_manager.towers.iter().map(|t| {
            SavedTower {
                tower_type: t.tower_type(),
                position: t.get_position(),
                level: t.get_level(),
                rotation: t.get_rotation(),
            }
        }).collect();

        let enemies = self.enemy_manager.get_enemies().iter().map(|e| {
            let path_index = if let Some(enemy_path) = e.get_path() {
                self.map.paths.iter().position(|p| p == enemy_path).unwrap_or(0)
            } else {
                0
            };
            SavedEnemy {
                enemy_type: e.enemy_type(),
                position: e.get_position(),
                health: e.get_health(),
                waypoint_index: e.get_waypoint_index(),
                path_index,
            }
        }).collect();

        let save_data = SaveData {
            map_name: self.current_map_name.clone(),
            money: self.economy.money,
            lives: self.player_stats.lives,
            max_lives: self.player_stats.max_lives,
            current_wave_index: self.wave_manager.current_wave_index,
            current_spawn_index: self.wave_manager.current_spawn_index,
            spawn_count_current_group: self.wave_manager.spawn_count_current_group,
            spawn_timer: self.wave_manager.spawn_timer,
            inter_wave_timer: self.wave_manager.inter_wave_timer,
            wave_state: self.wave_manager.state,
            enemy_spawn_count: self.enemy_spawn_count,
            continued_after_victory: self.continued_after_victory,
            towers,
            enemies,
        };

        save_data.save_to_file("isoguard_save.json")?;
        self.save_feedback = Some(("Game Saved Successfully!".to_string(), 2.0));
        log::info!("Game saved to isoguard_save.json");
        Ok(())
    }

    pub fn save_statistics(&self) {
        let mut save = crate::game::progress::SaveData::load();
        save.statistics = self.statistics.clone();
        save.save();
    }

    pub fn load_game(&mut self) -> Result<(), anyhow::Error> {
        self.is_loading_save = true;
        let result = self.load_game_impl();
        self.is_loading_save = false;
        result
    }

    fn load_game_impl(&mut self) -> Result<(), anyhow::Error> {
        use crate::game::save_game::SaveData;
        use crate::game::towers::basic_tower::BasicTower;
        use crate::game::towers::sniper_tower::SniperTower;
        use crate::game::enemies::basic_enemy::BasicEnemy;
        use crate::game::enemies::fast_enemy::FastEnemy;

        let save_data = SaveData::load_from_file("isoguard_save.json")?;

        // 1. Load the map (resets everything)
        self.load_level(&save_data.map_name);

        // 2. Restore economy and player stats
        self.economy.money = save_data.money;
        self.player_stats.lives = save_data.lives;
        self.player_stats.max_lives = save_data.max_lives;

        // 3. Restore wave manager properties
        self.wave_manager.current_wave_index = save_data.current_wave_index;
        self.wave_manager.current_spawn_index = save_data.current_spawn_index;
        self.wave_manager.spawn_count_current_group = save_data.spawn_count_current_group;
        self.wave_manager.spawn_timer = save_data.spawn_timer;
        self.wave_manager.inter_wave_timer = save_data.inter_wave_timer;
        self.wave_manager.state = save_data.wave_state;

        self.enemy_spawn_count = save_data.enemy_spawn_count;
        self.continued_after_victory = save_data.continued_after_victory;

        // Reset runtime structures
        self.enemy_manager = crate::game::enemies::EnemyManager::new();
        self.tower_manager = crate::game::towers::manager::TowerManager::new();
        self.projectiles.clear();
        self.popups.clear();
        self.particles.clear();
        self.selected_tower_index = None;

        // 4. Reconstruct towers
        for t in save_data.towers {
            let tower: Box<dyn crate::game::towers::tower_base::Tower> = match t.tower_type {
                crate::game::towers::manager::TowerType::Basic => {
                    let mut b = BasicTower::new(t.position);
                    b.level = t.level;
                    b.rotation = t.rotation;
                    Box::new(b)
                }
                crate::game::towers::manager::TowerType::Sniper => {
                    let mut s = SniperTower::new(t.position);
                    s.level = t.level;
                    s.rotation = t.rotation;
                    Box::new(s)
                }
            };
            self.tower_manager.towers.push(tower);
        }

        // 5. Reconstruct enemies
        for e in save_data.enemies {
            let path = self.map.paths.get(e.path_index).unwrap_or(&self.map.path);
            let enemy: Box<dyn crate::game::enemies::enemy_base::Enemy> = match e.enemy_type {
                crate::game::wave_manager::EnemyType::Basic => {
                    let mut b = BasicEnemy::new(e.position).with_path(path.clone());
                    b.health = e.health;
                    b.waypoint_index = e.waypoint_index;
                    Box::new(b)
                }
                crate::game::wave_manager::EnemyType::Fast => {
                    let mut f = FastEnemy::new(e.position).with_path(path.clone());
                    f.health = e.health;
                    f.waypoint_index = e.waypoint_index;
                    Box::new(f)
                }
            };
            self.enemy_manager.spawn_enemy(enemy);
        }

        // Make sure game state is set to playing
        self.game_state = crate::game::game_state::GameState::Playing;

        self.save_feedback = Some(("Game Loaded Successfully!".to_string(), 2.0));
        self.update_window_title();
        log::info!("Game loaded from isoguard_save.json");
        Ok(())
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

        // Update save/load feedback notification timer
        let dt = self.time.frame_delta_secs();
        if let Some((msg, timer)) = self.save_feedback.take() {
            let next_timer = timer - dt;
            if next_timer > 0.0 {
                self.save_feedback = Some((msg, next_timer));
            }
        }

        // Update store sliding animation
        if self.store_open {
            self.store_animation = (self.store_animation + dt * 6.0).min(1.0);
        } else {
            self.store_animation = (self.store_animation - dt * 6.0).max(0.0);
        }


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
        if self.game_state == crate::game::game_state::GameState::MainMenu {
            self.time_elapsed += dt;
            let angle = self.time_elapsed * 0.15; // Slow cinematic rotation
            let radius = 0.6; // Circular pan radius over procedural island
            self.camera.target = glam::Vec3::new(
                angle.cos() * radius,
                0.0,
                angle.sin() * radius,
            );
            return;
        }

        // Camera movement should work even in menus/paused states so the player can navigate.
        self.camera_controller.update_camera(&mut self.camera, dt);

        if self.game_state != crate::game::game_state::GameState::Playing {
            return;
        }

        if !self.player_stats.is_alive() {
            self.game_state = crate::game::game_state::GameState::GameOver;
            self.placement_status = "GAME OVER!".to_string();
            self.update_window_title();

            // Save final high score check for statistics
            let score = self.economy.money.max(0) as u32;
            self.statistics.high_score = self.statistics.high_score.max(score);
            self.save_statistics();
            return;
        }

        if self.wave_manager.state == crate::game::wave_manager::WaveState::CompletedAll && !self.continued_after_victory {
            self.game_state = crate::game::game_state::GameState::Victory;
            self.placement_status = "VICTORY!".to_string();
            self.update_window_title();

            // Calculate and save high score/progress
            let score = (self.player_stats.lives.max(0) as u32 * 100) + self.economy.money.max(0) as u32;
            
            self.statistics.wins += 1;
            self.statistics.high_score = self.statistics.high_score.max(score);

            let mut save = crate::game::progress::SaveData::load();
            match self.current_map_name.as_str() {
                "easy" => {
                    save.easy.completed = true;
                    save.easy.high_score = save.easy.high_score.max(score);
                    save.medium.unlocked = true;
                }
                "medium" => {
                    save.medium.completed = true;
                    save.medium.high_score = save.medium.high_score.max(score);
                    save.hard.unlocked = true;
                }
                "hard" | "volcanic" => {
                    save.hard.completed = true;
                    save.hard.high_score = save.hard.high_score.max(score);
                }
                _ => {}
            }
            save.statistics = self.statistics.clone();
            save.save();

            return;
        }

        self.time_elapsed += dt;
        
        let active_enemy_count = self.enemy_manager.get_enemies().len();
        let prev_state = self.wave_manager.state;
        let prev_wave_num = self.wave_manager.current_wave_number();
        let prev_timer = self.wave_manager.inter_wave_timer;

        if let Some(enemy_type) = self.wave_manager.update(dt, active_enemy_count) {
            let path_idx = self.enemy_spawn_count % self.map.paths.len().max(1);
            self.enemy_spawn_count += 1;
            let path = self.map.paths.get(path_idx).unwrap_or(&self.map.path);

            match enemy_type {
                crate::game::wave_manager::EnemyType::Basic => {
                    if let Some(start_pos) = path.start() {
                        let enemy = crate::game::enemies::basic_enemy::BasicEnemy::new(start_pos)
                            .with_path(path.clone());
                        self.enemy_manager.spawn_enemy(Box::new(enemy));
                    }
                }
                crate::game::wave_manager::EnemyType::Fast => {
                    if let Some(start_pos) = path.start() {
                        let enemy = crate::game::enemies::fast_enemy::FastEnemy::new(start_pos)
                            .with_path(path.clone());
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

        if state_changed {
            if prev_state == crate::game::wave_manager::WaveState::WaitingForClean
                && (self.wave_manager.state == crate::game::wave_manager::WaveState::InterWaveDelay
                    || self.wave_manager.state == crate::game::wave_manager::WaveState::CompletedAll)
            {
                self.statistics.waves_completed += 1;
                self.save_statistics();
            }
        }
        
        let (killed, escaped) = self.enemy_manager.update(dt, &self.map.path);

        if escaped > 0 {
            self.player_stats.take_damage(escaped as i32);
            if !self.player_stats.is_alive() {
                self.game_state = crate::game::game_state::GameState::GameOver;
                self.placement_status = "GAME OVER!".to_string();
                self.update_window_title();

                let score = self.economy.money.max(0) as u32;
                self.statistics.high_score = self.statistics.high_score.max(score);
                self.save_statistics();
                return;
            }
            self.update_window_title();
        }

        let killed_count = killed.len();
        if killed_count > 0 {
            self.statistics.enemies_killed += killed_count as u32;
        }

        for (pos, reward) in killed {
            self.economy.add_money(reward as i32);
            self.update_window_title();
            self.audio.play_explosion();
            
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

            // Spawn enemy death explosion
            self.particles.spawn_enemy_explosion(glam::Vec3::new(world_x, world_y + 0.02, world_z));
        }
        
        // Populate spatial grid with enemies for tower queries
        self.spatial_grid.clear();
        for (idx, enemy) in self.enemy_manager.get_enemies().iter().enumerate() {
            if enemy.is_alive() {
                self.spatial_grid.insert(idx, enemy.get_position());
            }
        }

        // Update towers and handle projectiles
        let new_projectiles = self.tower_manager.update_all(
            dt,
            self.enemy_manager.get_enemies(),
            Some(&self.spatial_grid),
            &mut self.query_scratch,
        );
        
        // Spawn muzzle flash for each new projectile
        for proj in &new_projectiles {
            self.audio.play_shoot();
            let start_gx = proj.start_position.x.round() as i32;
            let start_gy = proj.start_position.y.round() as i32;
            let start_tile_y = self.map.get_tile(start_gx, start_gy).map(|t| t.grid_y).unwrap_or(0);
            
            let start_world_x = (proj.start_position.x - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
            let start_world_z = (proj.start_position.y - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
            let start_world_y = (start_tile_y as f32 + 0.5 + proj.spawn_height_offset) * TILE_WORLD_SIZE;
            let muzzle_pos = glam::Vec3::new(start_world_x, start_world_y, start_world_z);

            let diff = proj.target_position - proj.start_position;
            let direction = if diff.length_squared() > 0.0001 { diff.normalize() } else { glam::Vec2::new(1.0, 0.0) };
            self.particles.spawn_muzzle_flash(muzzle_pos, direction);
        }
        
        self.projectiles.extend(new_projectiles);

        // Update active projectiles
        let mut to_remove = Vec::new();
        for (i, proj) in self.projectiles.iter_mut().enumerate() {
            let reached = proj.update(dt);
            
            // Calculate current 3D position
            let pos = proj.position;
            let world_x = (pos.x - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
            let world_z = (pos.y - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
            
            let start_gx = proj.start_position.x.round() as i32;
            let start_gy = proj.start_position.y.round() as i32;
            let start_tile_y = self.map.get_tile(start_gx, start_gy).map(|t| t.grid_y).unwrap_or(0);
            
            let target_gx = proj.target_position.x.round() as i32;
            let target_gy = proj.target_position.y.round() as i32;
            let target_tile_y = self.map.get_tile(target_gx, target_gy).map(|t| t.grid_y).unwrap_or(0);

            let start_world_y = (start_tile_y as f32 + 0.5 + proj.spawn_height_offset) * TILE_WORLD_SIZE;
            let target_world_y = (target_tile_y as f32 + 0.5) * TILE_WORLD_SIZE + 0.2 * TILE_WORLD_SIZE;
            
            let total_dist = proj.start_position.distance(proj.target_position);
            let current_dist = proj.position.distance(proj.target_position);
            let t_val = if total_dist > 0.0 { 1.0 - (current_dist / total_dist) } else { 1.0 };
            
            let world_y = start_world_y * (1.0 - t_val) + target_world_y * t_val;
            let proj_pos_3d = glam::Vec3::new(world_x, world_y, world_z);

            if reached {
                to_remove.push(i);
                
                // Spawn impact burst
                self.particles.spawn_impact_burst(proj_pos_3d);

                // Damage enemies near the target position
                for enemy in &mut self.enemy_manager.enemies {
                    let dist = enemy.get_position().distance(proj.target_position);
                    if dist < 0.5 { // 0.5 tile hit radius
                        enemy.take_damage(proj.damage);
                    }
                }
            } else {
                // Spawn trail particle
                let trail_color = if proj.speed >= 20.0 {
                    [0.2, 0.8, 1.0, 0.8] // Cyan/blue trail for sniper
                } else {
                    [1.0, 0.7, 0.1, 0.7] // Orange/yellow trail for basic
                };
                self.particles.spawn_projectile_trail(proj_pos_3d, trail_color);
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

        // Update particles
        self.particles.update(dt);
    }

    /// Per-frame render preparation: upload the camera matrix, resolve the
    /// hovered tile, and rebuild the dynamic draw batch. Runs once per rendered
    /// frame (variable rate), independent of the fixed logic steps above.
    fn prepare_frame(&mut self) {
        self.renderer.update_camera_uniform(&self.queue, &self.camera, self.config.width, self.config.height);

        // Precompute View-Projection and ortho dimension parameters for frustum culling
        let vp = self.camera.build_view_projection_matrix(self.config.width, self.config.height);
        let aspect = self.config.width as f32 / self.config.height as f32;
        let ortho_height = 2.0 / self.camera.zoom;
        let ortho_width = ortho_height * aspect;

        // Resolve which tile the cursor is over (camera may have moved, so this
        // is recomputed every frame).
        self.hovered_tile = self.pick_tile();

        // Clear the dynamic batcher for the new frame. Static terrain lives in
        // `terrain_batcher` and is NOT rebuilt here — only dynamic objects
        // (towers, enemies, projectiles) are queued each frame.
        self.batcher.clear();
        self.ui_batcher.clear();
        self.overlay_batcher.clear();

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
                    self.selected_tower_type,
                    &self.economy,
                ).is_ok();
                let highlight_color = if is_valid {
                    [0.0, 1.0, 0.0, 0.45] // Green for valid
                } else {
                    [1.0, 0.0, 0.0, 0.45] // Red for invalid
                };

                self.overlay_batcher.add_highlight_box(
                    glam::Vec3::new(pos_x, center_y, pos_z),
                    size,
                    highlight_color,
                    self.highlight_texture_id,
                );

                if self.selected_tower_index.is_none() {
                    let placement_range = match self.selected_tower_type {
                        crate::game::towers::manager::TowerType::Basic => 5.0,
                        crate::game::towers::manager::TowerType::Sniper => 12.0,
                    };
                    let range_y = center_y + TILE_WORLD_SIZE * 0.5 + 0.01;
                    self.overlay_batcher.add_overlay_circle(
                        glam::Vec3::new(pos_x, range_y, pos_z),
                        placement_range * TILE_WORLD_SIZE,
                        [1.0, 1.0, 1.0, 0.05], // Semi-transparent white
                        self.highlight_texture_id,
                    );
                }
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

            // Frustum Culling
            let world_pos = glam::Vec3::new(world_x, world_y, world_z);
            if !crate::renderer::camera::Camera::is_visible_with_vp(
                vp,
                world_pos,
                TILE_WORLD_SIZE,
                ortho_width,
                ortho_height,
            ) {
                continue;
            }

            // Apply scale and color tint based on upgrade level
            let level = tower.get_level();
            let (scale_multiplier, tint_color) = match level {
                1 => (1.0, [1.0, 1.0, 1.0, 1.0]),
                2 => (1.1, [0.75, 1.0, 1.0, 1.0]),  // Cyan-silver tint, 1.1x scale
                3 => (1.2, [1.0, 0.85, 0.3, 1.0]), // Gold tint, 1.2x scale
                _ => (1.0, [1.0, 1.0, 1.0, 1.0]),
            };

            let base_scale = match tower.tower_type() {
                crate::game::towers::manager::TowerType::Basic => TILE_WORLD_SIZE * 0.6,
                crate::game::towers::manager::TowerType::Sniper => TILE_WORLD_SIZE * 0.25,
            };
            let scale = base_scale * scale_multiplier;
            
            // BasicTower's rotation is an angle in the XY plane where +X is 0, +Y is PI/2.
            // In 3D, our camera XZ plane maps from 2D XY. So +Y in 2D is +Z in 3D.
            // A rotation of angle around Y axis from +X to +Z is exactly the same as 2D angle (if Y is UP).
            // Actually, in 3D Y is up, so rotation around Y from +X to +Z is a negative angle.
            let rot = tower.get_rotation();
            let mut rot_y = -rot;
            if let crate::game::towers::manager::TowerType::Sniper = tower.tower_type() {
                rot_y -= std::f32::consts::FRAC_PI_2;
            }

            let model = match tower.tower_type() {
                crate::game::towers::manager::TowerType::Basic => &self.tower_model,
                crate::game::towers::manager::TowerType::Sniper => &self.sniper_tower_model,
            };

            let vertices = model.generate_vertices(
                glam::Vec3::new(world_x, world_y, world_z),
                scale,
                rot_y,
                tint_color,
            );
            
            let tex_id = self.highlight_texture_id;
            self.batcher.add_model(vertices, tex_id);
        }

        // Draw selection highlight and range indicators for selected tower
        if let Some(idx) = self.selected_tower_index {
            if let Some(tower) = self.tower_manager.towers.get(idx) {
                let pos = tower.get_position();
                let gx = pos.x.round() as i32;
                let gy = pos.y.round() as i32;
                if let Some(tile) = self.map.get_tile(gx, gy) {
                    let pos_x = (gx as f32 - self.map.width as f32 / 2.0) * TILE_WORLD_SIZE;
                    let pos_z = (gy as f32 - self.map.height as f32 / 2.0) * TILE_WORLD_SIZE;
                    let center_y = tile.grid_y as f32 * TILE_WORLD_SIZE;
                    
                    // Render selected tower highlight box (blue outline)
                    let size = glam::Vec3::splat(TILE_WORLD_SIZE + HIGHLIGHT_INFLATE);
                    self.overlay_batcher.add_highlight_box(
                        glam::Vec3::new(pos_x, center_y, pos_z),
                        size,
                        [0.0, 0.6, 1.0, 0.45], // Blue highlight
                        self.highlight_texture_id,
                    );
                    
                    // Render range indicator circle on the ground
                    let range_radius = tower.get_range();
                    let range_y = center_y + TILE_WORLD_SIZE * 0.5 + 0.01;
                    
                    self.overlay_batcher.add_overlay_circle(
                        glam::Vec3::new(pos_x, range_y, pos_z),
                        range_radius * TILE_WORLD_SIZE,
                        [1.0, 1.0, 1.0, 0.05], // Semi-transparent white
                        self.highlight_texture_id,
                    );
                }
            }
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

            // Frustum Culling (radius includes enemy scale and health bar height offset)
            let scale = enemy.get_scale();
            let world_pos = glam::Vec3::new(world_x, world_y, world_z);
            if !crate::renderer::camera::Camera::is_visible_with_vp(
                vp,
                world_pos,
                scale * 2.0,
                ortho_width,
                ortho_height,
            ) {
                continue;
            }

            // Draw enemy model
            let model = match enemy.enemy_type() {
                crate::game::wave_manager::EnemyType::Basic => &self.enemy_model,
                crate::game::wave_manager::EnemyType::Fast => &self.fast_enemy_model,
            };
            let scale = enemy.get_scale();
            let color = enemy.get_color();
            let vertices = model.generate_vertices(
                glam::Vec3::new(world_x, world_y, world_z),
                scale,
                self.time_elapsed * 2.0, // rotation for visual effect
                color,
            );
            self.batcher.add_model(vertices, self.highlight_texture_id);

            // Draw health bar if damaged and alive
            let max_health = enemy.get_max_health();
            let health_pct = (health / max_health).clamp(0.0, 1.0);
            
            if health_pct < 1.0 && health > 0.0 {
                // Scale bar dimensions dynamically with camera zoom to keep it readable when zoomed out
                let scale_factor = (1.0 / self.camera.zoom).sqrt().clamp(1.0, 2.5);
                let bar_width = 0.08 * scale_factor;
                let bar_height = 0.01 * scale_factor;
                let border_width = bar_width + 0.006 * scale_factor;
                let border_height = bar_height + 0.004 * scale_factor;
                let bar_y = world_y + scale + 0.01 * scale_factor; // Dynamic height above the enemy

                let center_pos = glam::Vec3::new(world_x, bar_y, world_z);
                let right = glam::Vec3::new(1.0, 0.0, -1.0).normalize();
                let towards_camera = glam::Vec3::new(1.0, 1.0, 1.0).normalize();

                // 1. Black border
                self.batcher.add_sprite(
                    crate::renderer::sprite::Sprite::new_colored(
                        center_pos,
                        glam::Vec2::new(border_width, border_height),
                        self.highlight_texture_id,
                        0.0,
                        [0.0, 0.0, 0.0, 1.0], // Black outline
                    ),
                    crate::renderer::sprite::SpriteAlignment::Billboard,
                );

                // 2. Dark crimson background (missing health)
                let bg_pos = center_pos + towards_camera * 0.001;
                self.batcher.add_sprite(
                    crate::renderer::sprite::Sprite::new_colored(
                        bg_pos,
                        glam::Vec2::new(bar_width, bar_height),
                        self.highlight_texture_id,
                        0.0,
                        [0.3, 0.05, 0.05, 1.0], // Dark crimson
                    ),
                    crate::renderer::sprite::SpriteAlignment::Billboard,
                );

                // 3. Current health foreground
                let current_bar_width = bar_width * health_pct;
                let offset = right * (bar_width - current_bar_width) / 2.0;
                let fg_pos = center_pos + towards_camera * 0.002 - offset;

                // Color interpolation: green (100% health) -> yellow (50% health) -> red (0% health)
                let r = if health_pct > 0.5 {
                    2.0 * (1.0 - health_pct)
                } else {
                    1.0
                };
                let g = if health_pct > 0.5 {
                    1.0
                } else {
                    2.0 * health_pct
                };
                let b = 0.0;
                let foreground_color = [r, g, b, 1.0];

                self.batcher.add_sprite(
                    crate::renderer::sprite::Sprite::new_colored(
                        fg_pos,
                        glam::Vec2::new(current_bar_width, bar_height),
                        self.highlight_texture_id,
                        0.0,
                        foreground_color,
                    ),
                    crate::renderer::sprite::SpriteAlignment::Billboard,
                );
            }
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

            // Frustum Culling
            let world_pos = glam::Vec3::new(world_x, world_y, world_z);
            if !crate::renderer::camera::Camera::is_visible_with_vp(
                vp,
                world_pos,
                0.1,
                ortho_width,
                ortho_height,
            ) {
                continue;
            }
            
            // The generated sphere has radius 1.0, so let's scale it to 0.008
            let scale = 0.008; 
            let vertices = self.projectile_model.generate_vertices(
                world_pos,
                scale,
                0.0,
                [1.0, 1.0, 0.0, 1.0], // Yellow
            );
            self.batcher.add_model(vertices, self.highlight_texture_id);
        }

        // Draw popups
        for popup in &self.popups {
            // Frustum Culling
            if !crate::renderer::camera::Camera::is_visible_with_vp(
                vp,
                popup.position,
                0.2,
                ortho_width,
                ortho_height,
            ) {
                continue;
            }

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

        // Draw particles with frustum culling
        self.particles.draw(&mut self.batcher, self.highlight_texture_id, Some((vp, ortho_width, ortho_height)));

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
        if self.game_state != crate::game::game_state::GameState::Playing
            && self.game_state != crate::game::game_state::GameState::Paused
            && self.game_state != crate::game::game_state::GameState::MainMenu
        {
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
        self.overlay_batcher.finalize(&self.device, &self.queue);
    }

    /// Queues translucent ground markers tracing the map's enemy path into the
    /// dynamic batch. Markers are sampled along each segment so any path shape
    /// (including diagonals) is visualised.
    fn draw_path_debug(&mut self) {
        let paths = if self.map.paths.is_empty() {
            vec![self.map.path.clone()]
        } else {
            self.map.paths.clone()
        };

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

        for path in paths {
            let waypoints: Vec<glam::Vec2> = path.waypoints().to_vec();
            if waypoints.len() < 2 {
                continue;
            }

            for pair in waypoints.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let steps = (a.distance(b) / 0.15).ceil().max(1.0) as i32;
                for i in 0..=steps {
                    let p = a.lerp(b, i as f32 / steps as f32);
                    let (wx, wz) = to_world(p);
                    self.overlay_batcher.add_overlay_quad(
                        glam::Vec3::new(wx, y, wz),
                        size,
                        PATH_DEBUG_COLOR,
                        tex,
                    );
                }
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

        // Draw scene overlays (depth tested but depth writing disabled).
        self.renderer.render_overlay_batches(
            &view,
            &self.device,
            &self.queue,
            &[&self.overlay_batcher],
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

        // Apply pixel art styling (blocky corners) for the HUD
        let mut visuals = self.egui_ctx.style().visuals.clone();
        visuals.widgets.inactive.corner_radius = 0.into();
        visuals.widgets.hovered.corner_radius = 0.into();
        visuals.widgets.active.corner_radius = 0.into();
        visuals.window_corner_radius = 0.into();
        self.egui_ctx.set_visuals(visuals);

        // 2. Draw HUD if in GameState::Playing or GameState::Paused
        if self.game_state == crate::game::game_state::GameState::Playing
            || self.game_state == crate::game::game_state::GameState::Paused
        {
            // Draw top-left panel (health, gold, settings) - floating but blocky
            egui::Area::new(egui::Id::new("hud_top_left"))
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(20.0, 20.0))
                .show(&self.egui_ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(200))
                        .corner_radius(0.0)
                        .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(215, 115, 0)))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("❤️").size(22.0));
                                ui.label(egui::RichText::new(format!("{}", self.player_stats.lives))
                                    .font(egui::FontId::monospace(18.0))
                                    .color(egui::Color32::WHITE)
                                    .strong());
                                
                                ui.add_space(12.0);
                                ui.separator();
                                ui.add_space(12.0);
                                
                                ui.label(egui::RichText::new("💰").size(22.0));
                                ui.label(egui::RichText::new(format!("{}", self.economy.money))
                                    .font(egui::FontId::monospace(18.0))
                                    .color(egui::Color32::WHITE)
                                    .strong());
                                
                                ui.add_space(12.0);
                                ui.separator();
                                ui.add_space(12.0);
                                
                                let settings_btn = egui::Button::new(
                                    egui::RichText::new("⚙")
                                        .font(egui::FontId::monospace(18.0))
                                        .color(egui::Color32::WHITE)
                                )
                                .fill(egui::Color32::from_rgb(70, 70, 70))
                                .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 150, 150)))
                                .min_size(egui::vec2(28.0, 28.0));
                                
                                if ui.add(settings_btn).clicked() {
                                    self.show_ingame_settings = !self.show_ingame_settings;
                                    self.audio.play_click();
                                }
                            });
                        });
                });

            // Draw wave panel (top-center) - floating but blocky
            egui::Area::new(egui::Id::new("hud_wave"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 20.0))
                .show(&self.egui_ctx, |ui| {
                    let wave_text = match self.wave_manager.state {
                        crate::game::wave_manager::WaveState::NotStarted => {
                            let mut txt = "Waiting to Start".to_string();
                            if self.game_state == crate::game::game_state::GameState::Paused {
                                txt.push_str(" [PAUSED]");
                            }
                            txt
                        }
                        crate::game::wave_manager::WaveState::Spawning | crate::game::wave_manager::WaveState::WaitingForClean => {
                            let mut txt = format!("Wave {}/{}", self.wave_manager.current_wave_number(), self.wave_manager.waves.len());
                            if self.game_state == crate::game::game_state::GameState::Paused {
                                txt.push_str(" [PAUSED]");
                            }
                            txt
                        }
                        crate::game::wave_manager::WaveState::InterWaveDelay => {
                            if self.wave_manager.inter_wave_timer > 7.0 {
                                let mut txt = format!("Wave {} Complete!", self.wave_manager.current_wave_number());
                                if self.game_state == crate::game::game_state::GameState::Paused {
                                    txt.push_str(" [PAUSED]");
                                }
                                txt
                            } else {
                                let next_wave = self.wave_manager.current_wave_number() + 1;
                                let mut txt = format!("Wave {} Incoming: {:.1}s", next_wave, self.wave_manager.inter_wave_timer);
                                if self.game_state == crate::game::game_state::GameState::Paused {
                                    txt.push_str(" [PAUSED]");
                                }
                                txt
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
                        .corner_radius(0.0)
                        .stroke(egui::Stroke::new(3.0, border_color))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚔️").size(24.0));
                                ui.label(egui::RichText::new(wave_text)
                                    .font(egui::FontId::monospace(20.0))
                                    .color(egui::Color32::WHITE)
                                    .strong());
                            });
                        });
                });

            // Draw tower shop panel (sliding along the bottom-left edge, NOT floating)
            let panel_width = 360.0;
            let slide_distance = panel_width;
            let offset_x = -slide_distance + (self.store_animation * slide_distance);
            
            if self.store_animation > 0.0 {
                egui::Area::new(egui::Id::new("hud_shop"))
                    .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(offset_x, 0.0)) // Flat on bottom!
                    .show(&self.egui_ctx, |ui| {
                        egui::Frame::NONE
                            .fill(egui::Color32::from_black_alpha(190))
                            .corner_radius(0.0) // Flat retro corner
                            .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(215, 115, 0))) // Blocky orange border
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_width(336.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("🛠️ SHOP")
                                            .font(egui::FontId::monospace(14.0))
                                            .color(egui::Color32::from_rgb(215, 115, 0))
                                            .strong()
                                    );
                                    ui.add_space(8.0);
                                    
                                    ui.horizontal(|ui| {
                                        // 1. Basic Tower Button
                                        let basic_selected = self.selected_tower_type == crate::game::towers::manager::TowerType::Basic;
                                        let basic_cost = crate::game::towers::manager::TowerType::Basic.cost();
                                        let can_afford_basic = self.economy.money >= basic_cost;
                                        
                                        let basic_border = if basic_selected {
                                            egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 100))
                                        } else {
                                            egui::Stroke::new(1.5, egui::Color32::GRAY)
                                        };
                                        
                                        let basic_bg = if basic_selected {
                                            egui::Color32::from_rgb(20, 50, 30)
                                        } else {
                                            egui::Color32::from_black_alpha(100)
                                        };
                                        
                                        let basic_btn = egui::Button::new(
                                            egui::RichText::new(format!("🏹 Basic Tower\nCost: {}g", basic_cost))
                                                .font(egui::FontId::monospace(15.0))
                                                .color(if can_afford_basic { egui::Color32::WHITE } else { egui::Color32::from_rgb(255, 100, 100) })
                                                .strong()
                                        )
                                        .fill(basic_bg)
                                        .stroke(basic_border)
                                        .min_size(egui::vec2(160.0, 48.0));
                                        
                                        if ui.add(basic_btn).on_hover_text("Range: 5.0 | DPS: 25.0\nStandard general-purpose defensive tower.").clicked() {
                                            self.selected_tower_type = crate::game::towers::manager::TowerType::Basic;
                                            self.audio.play_click();
                                            log::info!("Selected Basic Tower for placement");
                                        }
                                        
                                        ui.add_space(16.0);
                                        
                                        // 2. Sniper Tower Button
                                        let sniper_selected = self.selected_tower_type == crate::game::towers::manager::TowerType::Sniper;
                                        let sniper_cost = crate::game::towers::manager::TowerType::Sniper.cost();
                                        let can_afford_sniper = self.economy.money >= sniper_cost;
                                        
                                        let sniper_border = if sniper_selected {
                                            egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 100))
                                        } else {
                                            egui::Stroke::new(1.5, egui::Color32::GRAY)
                                        };
                                        
                                        let sniper_bg = if sniper_selected {
                                            egui::Color32::from_rgb(20, 50, 30)
                                        } else {
                                            egui::Color32::from_black_alpha(100)
                                        };
                                        
                                        let sniper_btn = egui::Button::new(
                                            egui::RichText::new(format!("🎯 Sniper Tower\nCost: {}g", sniper_cost))
                                                .font(egui::FontId::monospace(15.0))
                                                .color(if can_afford_sniper { egui::Color32::WHITE } else { egui::Color32::from_rgb(255, 100, 100) })
                                                .strong()
                                        )
                                        .fill(sniper_bg)
                                        .stroke(sniper_border)
                                        .min_size(egui::vec2(160.0, 48.0));
                                        
                                        if ui.add(sniper_btn).on_hover_text("Range: 12.0 | Damage: 100.0\nSlow firing rate, but deals heavy damage over long distances.").clicked() {
                                            self.selected_tower_type = crate::game::towers::manager::TowerType::Sniper;
                                            self.audio.play_click();
                                            log::info!("Selected Sniper Tower for placement");
                                        }
                                    });
                                });
                            });
                    });
            }

            // Draw store toggle button (flat on the bottom, slides in sync)
            let button_x = self.store_animation * panel_width;
            egui::Area::new(egui::Id::new("hud_shop_toggle"))
                .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(button_x, 0.0)) // Flat on bottom!
                .show(&self.egui_ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(200))
                        .corner_radius(0.0) // Flat retro corner
                        .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(215, 115, 0))) // Blocky orange border
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            let text = if self.store_open { "◀ Shop" } else { "🛒 Shop" };
                            let toggle_btn = egui::Button::new(
                                egui::RichText::new(text)
                                    .font(egui::FontId::monospace(16.0))
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            )
                            .fill(egui::Color32::TRANSPARENT);
                            
                            if ui.add(toggle_btn).clicked() {
                                self.store_open = !self.store_open;
                                self.audio.play_click();
                            }
                        });
                });

            // Draw speed controls panel (bottom-right, flat on bottom)
            egui::Area::new(egui::Id::new("hud_speed_controls"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, 0.0)) // Flat on bottom, offset 20.0 from right
                .show(&self.egui_ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(200))
                        .corner_radius(0.0) // Flat retro corner
                        .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(215, 115, 0))) // Blocky orange border
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // 1. Pause button
                                let is_paused = self.game_state == crate::game::game_state::GameState::Paused;
                                let pause_border = if is_paused {
                                    egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 255, 100))
                                } else {
                                    egui::Stroke::new(1.5, egui::Color32::GRAY)
                                };
                                let pause_bg = if is_paused {
                                    egui::Color32::from_rgb(20, 50, 30)
                                } else {
                                    egui::Color32::from_black_alpha(100)
                                };
                                let pause_btn = egui::Button::new(
                                    egui::RichText::new(if is_paused { "▶ Resume" } else { "⏸ Pause" })
                                        .font(egui::FontId::monospace(14.0))
                                        .color(egui::Color32::WHITE)
                                        .strong()
                                )
                                .fill(pause_bg)
                                .stroke(pause_border)
                                .min_size(egui::vec2(80.0, 32.0));
                                
                                if ui.add(pause_btn).clicked() {
                                    self.audio.play_click();
                                    if is_paused {
                                        self.game_state = crate::game::game_state::GameState::Playing;
                                    } else {
                                        self.game_state = crate::game::game_state::GameState::Paused;
                                    }
                                }
                                
                                ui.add_space(8.0);
                                
                                // 2. 0.5x Slow button
                                let is_slow = !is_paused && self.time.speed_multiplier == 0.5;
                                let slow_border = if is_slow {
                                    egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 255, 100))
                                } else {
                                    egui::Stroke::new(1.5, egui::Color32::GRAY)
                                };
                                let slow_bg = if is_slow {
                                    egui::Color32::from_rgb(20, 50, 30)
                                } else {
                                    egui::Color32::from_black_alpha(100)
                                };
                                let slow_btn = egui::Button::new(
                                    egui::RichText::new("🐢 0.5x")
                                        .font(egui::FontId::monospace(14.0))
                                        .color(egui::Color32::WHITE)
                                        .strong()
                                )
                                .fill(slow_bg)
                                .stroke(slow_border)
                                .min_size(egui::vec2(60.0, 32.0));
                                
                                if ui.add(slow_btn).clicked() {
                                    self.audio.play_click();
                                    self.game_state = crate::game::game_state::GameState::Playing;
                                    self.time.speed_multiplier = 0.5;
                                }
                                
                                ui.add_space(4.0);
                                
                                // 3. 1.0x Normal button
                                let is_normal = !is_paused && self.time.speed_multiplier == 1.0;
                                let normal_border = if is_normal {
                                    egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 255, 100))
                                } else {
                                    egui::Stroke::new(1.5, egui::Color32::GRAY)
                                };
                                let normal_bg = if is_normal {
                                    egui::Color32::from_rgb(20, 50, 30)
                                } else {
                                    egui::Color32::from_black_alpha(100)
                                };
                                let normal_btn = egui::Button::new(
                                    egui::RichText::new("▶ 1.0x")
                                        .font(egui::FontId::monospace(14.0))
                                        .color(egui::Color32::WHITE)
                                        .strong()
                                )
                                .fill(normal_bg)
                                .stroke(normal_border)
                                .min_size(egui::vec2(60.0, 32.0));
                                
                                if ui.add(normal_btn).clicked() {
                                    self.audio.play_click();
                                    self.game_state = crate::game::game_state::GameState::Playing;
                                    self.time.speed_multiplier = 1.0;
                                }
                                
                                ui.add_space(4.0);
                                
                                // 4. 2.0x Fast button
                                let is_fast = !is_paused && self.time.speed_multiplier == 2.0;
                                let fast_border = if is_fast {
                                    egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 255, 100))
                                } else {
                                    egui::Stroke::new(1.5, egui::Color32::GRAY)
                                };
                                let fast_bg = if is_fast {
                                    egui::Color32::from_rgb(20, 50, 30)
                                } else {
                                    egui::Color32::from_black_alpha(100)
                                };
                                let fast_btn = egui::Button::new(
                                    egui::RichText::new("⚡ 2.0x")
                                        .font(egui::FontId::monospace(14.0))
                                        .color(egui::Color32::WHITE)
                                        .strong()
                                )
                                .fill(fast_bg)
                                .stroke(fast_border)
                                .min_size(egui::vec2(60.0, 32.0));
                                
                                if ui.add(fast_btn).clicked() {
                                    self.audio.play_click();
                                    self.game_state = crate::game::game_state::GameState::Playing;
                                    self.time.speed_multiplier = 2.0;
                                }
                            });
                        });
                });
        }

        // Draw Save/Load feedback notification
        if let Some((msg, _)) = &self.save_feedback {
            let egui_ctx = self.egui_ctx.clone();
            egui::Area::new(egui::Id::new("hud_save_feedback"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 40.0))
                .show(&egui_ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(220))
                        .corner_radius(8.0)
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 230, 255)))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(msg)
                                    .font(egui::FontId::proportional(16.0))
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            );
                        });
                });
        }

        // Draw Selected Tower Info / Upgrade Panel (right-center)
        let mut upgrade_triggered = false;
        let mut close_triggered = false;
        
        if self.game_state == crate::game::game_state::GameState::Playing
            || self.game_state == crate::game::game_state::GameState::Paused
        {
            if let Some(idx) = self.selected_tower_index {
                if let Some(tower) = self.tower_manager.towers.get(idx) {
                    let tower_type = tower.tower_type();
                    let level = tower.get_level();
                    let current_damage = tower.get_damage();
                    let current_range = tower.get_range();
                    let current_fire_rate = tower.get_fire_rate();
                    
                    let (can_upgrade, upgrade_cost) = if level < 3 {
                        if let Some(cost) = tower.get_upgrade_cost() {
                            (true, cost)
                        } else {
                            (false, 0)
                        }
                    } else {
                        (false, 0)
                    };
                    let can_afford_upgrade = self.economy.money >= upgrade_cost;

                    egui::Area::new(egui::Id::new("hud_upgrade_panel"))
                        .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-20.0, 0.0))
                        .show(&self.egui_ctx, |ui| {
                            egui::Frame::NONE
                                .fill(egui::Color32::from_black_alpha(200))
                                .corner_radius(12.0)
                                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 180, 255)))
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_max_width(260.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("🏰 SELECTED TOWER")
                                                .font(egui::FontId::proportional(16.0))
                                                .color(egui::Color32::from_rgb(0, 180, 255))
                                                .strong()
                                        );
                                        ui.add_space(8.0);
                                        
                                        ui.label(
                                            egui::RichText::new(format!("{:?} (Level {})", tower_type, level))
                                                .font(egui::FontId::proportional(18.0))
                                                .color(egui::Color32::WHITE)
                                                .strong()
                                        );
                                        
                                        ui.add_space(8.0);
                                        ui.separator();
                                        ui.add_space(8.0);
                                        
                                        // Current Stats
                                        ui.label(egui::RichText::new("Current Stats:").strong().color(egui::Color32::LIGHT_GRAY));
                                        ui.label(format!("• Damage: {}", current_damage));
                                        ui.label(format!("• Range: {:.1}", current_range));
                                        ui.label(format!("• Attack Speed: {:.2}/s", current_fire_rate));
                                        
                                        ui.add_space(12.0);
                                        
                                        if can_upgrade {
                                            // Stats after upgrade
                                            let next_lvl = level + 1;
                                            let (dmg_inc, rng_inc) = match next_lvl {
                                                2 => (0.50, 0.25),
                                                3 => (1.00, 0.50),
                                                _ => (0.0, 0.0),
                                            };
                                            let base_dmg = match tower_type {
                                                crate::game::towers::manager::TowerType::Basic => 25.0,
                                                crate::game::towers::manager::TowerType::Sniper => 100.0,
                                            };
                                            let base_rng = match tower_type {
                                                crate::game::towers::manager::TowerType::Basic => 5.0,
                                                crate::game::towers::manager::TowerType::Sniper => 12.0,
                                            };
                                            let next_damage = base_dmg * (1.0 + dmg_inc);
                                            let next_range = base_rng * (1.0 + rng_inc);

                                            ui.label(egui::RichText::new("Next Level Stats:").strong().color(egui::Color32::from_rgb(0, 255, 100)));
                                            ui.label(format!("• Damage: {} ( +{} )", next_damage, next_damage - current_damage));
                                            ui.label(format!("• Range: {:.1} ( +{:.1} )", next_range, next_range - current_range));
                                            
                                            ui.add_space(16.0);
                                            
                                            let upgrade_text = format!("✨ Upgrade ({}g)", upgrade_cost);
                                            let upgrade_btn = egui::Button::new(
                                                egui::RichText::new(upgrade_text)
                                                    .font(egui::FontId::proportional(16.0))
                                                    .color(egui::Color32::WHITE)
                                                    .strong()
                                            )
                                            .fill(if can_afford_upgrade { egui::Color32::from_rgb(46, 125, 50) } else { egui::Color32::from_rgb(120, 40, 40) })
                                            .stroke(egui::Stroke::new(1.5, if can_afford_upgrade { egui::Color32::from_rgb(102, 187, 106) } else { egui::Color32::from_rgb(200, 100, 100) }));
                                            
                                            let response = ui.add_sized([220.0, 40.0], upgrade_btn);
                                            if !can_afford_upgrade {
                                                response.on_hover_text("Insufficient gold!");
                                            } else if response.clicked() {
                                                upgrade_triggered = true;
                                                self.audio.play_click();
                                            }
                                        } else {
                                            ui.label(
                                                egui::RichText::new("⭐ MAX LEVEL REACHED")
                                                    .font(egui::FontId::proportional(14.0))
                                                    .color(egui::Color32::from_rgb(255, 220, 0))
                                                    .strong()
                                            );
                                        }
                                        
                                        ui.add_space(12.0);
                                        
                                        // Close button
                                        let close_btn = egui::Button::new(
                                            egui::RichText::new("Close")
                                                .font(egui::FontId::proportional(14.0))
                                                .color(egui::Color32::WHITE)
                                        )
                                        .fill(egui::Color32::from_black_alpha(100))
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY));
                                        
                                        if ui.add_sized([220.0, 28.0], close_btn).clicked() {
                                            close_triggered = true;
                                            self.audio.play_click();
                                        }
                                    });
                                });
                        });
                }
            }
        }

        if upgrade_triggered {
            if let Some(idx) = self.selected_tower_index {
                if let Some(tower) = self.tower_manager.towers.get_mut(idx) {
                    let tower_type = tower.tower_type();
                    let current_level = tower.get_level();
                    if let Some(cost) = tower.get_upgrade_cost() {
                        if self.economy.purchase(cost) {
                            if let Ok(_) = tower.upgrade() {
                                self.placement_status = format!("Upgraded {:?} to Level {}", tower_type, current_level + 1);
                                self.update_window_title();
                                log::info!("Upgraded tower at index {} to level {}", idx, current_level + 1);
                            }
                        }
                    }
                }
            }
        }

        if close_triggered {
            self.selected_tower_index = None;
            self.placement_status = "Ready".to_string();
            self.update_window_title();
        }

        // Draw in-game settings modal if open
        if self.show_ingame_settings {
            let mut show = true;
            let egui_ctx = self.egui_ctx.clone();
            egui::Window::new("⚙️  SETTINGS")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .resizable(false)
                .collapsible(false)
                .open(&mut show)
                .show(&egui_ctx, |ui| {
                    ui.set_width(280.0);
                    ui.add_space(8.0);
                    
                    // SFX Volume Slider
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔊 SFX Vol:").color(egui::Color32::WHITE).font(egui::FontId::monospace(14.0)));
                        let mut sfx_vol = self.audio.get_volume();
                        if ui.add(egui::Slider::new(&mut sfx_vol, 0.0..=1.0)).changed() {
                            self.audio.set_volume(sfx_vol);
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // Music Volume Slider
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎵 Music Vol:").color(egui::Color32::WHITE).font(egui::FontId::monospace(14.0)));
                        let mut music_vol = self.audio.get_music_volume();
                        if ui.add(egui::Slider::new(&mut music_vol, 0.0..=1.0)).changed() {
                            self.audio.set_music_volume(music_vol);
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // Music Mute Checkbox
                    let mut music_muted = self.audio.get_music_muted();
                    if ui.checkbox(&mut music_muted, egui::RichText::new("Mute Music").color(egui::Color32::WHITE).font(egui::FontId::monospace(14.0))).changed() {
                        self.audio.set_music_muted(music_muted);
                        self.audio.play_click();
                    }
                    
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);
                    
                    let btn_width = 240.0;
                    let btn_height = 32.0;
                    
                    // Restart button
                    let restart_btn = egui::Button::new(
                        egui::RichText::new("🔄  Restart Game")
                            .font(egui::FontId::monospace(14.0))
                            .color(egui::Color32::WHITE)
                            .strong()
                    )
                    .fill(egui::Color32::from_rgb(21, 101, 192))
                    .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 181, 246)));
                    
                    if ui.add_sized([btn_width, btn_height], restart_btn).clicked() {
                        self.start_game();
                        self.audio.play_click();
                        self.show_ingame_settings = false;
                        log::info!("Restarted game via in-game Settings");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Save Game button
                    let save_btn = egui::Button::new(
                        egui::RichText::new("💾  Save Game")
                            .font(egui::FontId::monospace(14.0))
                            .color(egui::Color32::WHITE)
                            .strong()
                    )
                    .fill(egui::Color32::from_rgb(136, 14, 79))
                    .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(244, 143, 177)));
                    
                    if ui.add_sized([btn_width, btn_height], save_btn).clicked() {
                        if let Err(e) = self.save_game() {
                            self.save_feedback = Some((format!("Save failed: {}", e), 3.0));
                        }
                        self.audio.play_click();
                    }
                    
                    ui.add_space(8.0);
                    
                    // Load Game button
                    let save_exists = std::path::Path::new("isoguard_save.json").exists();
                    ui.add_enabled_ui(save_exists, |ui| {
                        let load_btn = egui::Button::new(
                            egui::RichText::new("📂  Load Game")
                                .font(egui::FontId::monospace(14.0))
                                .strong()
                        )
                        .fill(egui::Color32::from_rgb(74, 20, 140))
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(186, 104, 200)));
                        
                        if ui.add_sized([btn_width, btn_height], load_btn).clicked() {
                            if let Err(e) = self.load_game() {
                                self.save_feedback = Some((format!("Load failed: {}", e), 3.0));
                            } else {
                                self.game_state = crate::game::game_state::GameState::Playing;
                                self.show_ingame_settings = false;
                            }
                            self.audio.play_click();
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // Quit to Main Menu button
                    let quit_btn = egui::Button::new(
                        egui::RichText::new("🚪  Exit to Main Menu")
                            .font(egui::FontId::monospace(14.0))
                            .color(egui::Color32::WHITE)
                            .strong()
                    )
                    .fill(egui::Color32::from_rgb(198, 40, 40))
                    .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(239, 83, 80)));
                    
                    if ui.add_sized([btn_width, btn_height], quit_btn).clicked() {
                        self.game_state = crate::game::game_state::GameState::MainMenu;
                        self.audio.play_click();
                        self.show_ingame_settings = false;
                        log::info!("Exited to Main Menu");
                    }

                    ui.add_space(12.0);
                });
            if !show {
                self.show_ingame_settings = false;
            }
        }

        // Draw Main Menu if in GameState::MainMenu
        if self.game_state == crate::game::game_state::GameState::MainMenu {
            let egui_ctx = self.egui_ctx.clone();
            
            if self.show_level_select {
                // Horizontal Scroll Map Level Selection
                struct LevelInfo {
                    name: &'static str,
                    difficulty: &'static str,
                    biome: &'static str,
                    biome_emoji: &'static str,
                    description: &'static str,
                    map_id: &'static str,
                    unlocked: bool,
                    completed: bool,
                    high_score: u32,
                    grid: [[u32; 10]; 10],
                }

                let save = crate::game::progress::SaveData::load();

                let levels = vec![
                    LevelInfo {
                        name: "Whispering Woods",
                        difficulty: "EASY",
                        biome: "Grasslands",
                        biome_emoji: "🌲",
                        description: "A peaceful valley surrounded by dense forests. Perfect for establishing basic defensive perimeters along the straight road.",
                        map_id: "easy",
                        unlocked: true,
                        completed: save.easy.completed,
                        high_score: save.easy.high_score,
                        grid: [
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                            [0, 2, 2, 0, 0, 0, 2, 2, 0, 0],
                            [0, 2, 0, 0, 0, 0, 0, 2, 0, 0],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                            [0, 2, 0, 0, 0, 0, 0, 2, 0, 0],
                            [0, 2, 2, 0, 0, 0, 2, 2, 0, 0],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
                        ],
                    },
                    LevelInfo {
                        name: "Dust Devil Canyon",
                        difficulty: "MEDIUM",
                        biome: "Desert Dunes",
                        biome_emoji: "🏜️",
                        description: "A winding, hot canyon where the desert winds howl. The path bends sharply, giving you multiple angles to mount turret defenses.",
                        map_id: "medium",
                        unlocked: save.medium.unlocked,
                        completed: save.medium.completed,
                        high_score: save.medium.high_score,
                        grid: [
                            [0, 0, 0, 0, 2, 0, 0, 0, 0, 0],
                            [1, 1, 1, 1, 1, 0, 0, 2, 0, 0],
                            [0, 2, 0, 0, 1, 0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 1, 2, 0, 0, 0, 0],
                            [2, 0, 0, 0, 1, 0, 0, 0, 2, 0],
                            [0, 1, 1, 1, 1, 0, 0, 0, 0, 0],
                            [0, 1, 2, 0, 0, 0, 2, 0, 0, 0],
                            [0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                            [0, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                            [0, 0, 0, 2, 0, 0, 0, 2, 0, 0]
                        ],
                    },
                    LevelInfo {
                        name: "Frostbite Pass",
                        difficulty: "HARD",
                        biome: "Snowy Tundra",
                        biome_emoji: "❄️",
                        description: "An icy corridor flanked by frozen peaks. The subzero temperatures freeze your veins, and enemies arrive from multiple directions.",
                        map_id: "hard",
                        unlocked: save.hard.unlocked,
                        completed: save.hard.completed,
                        high_score: save.hard.high_score,
                        grid: [
                            [0, 2, 0, 0, 0, 0, 0, 2, 0, 0],
                            [0, 0, 0, 2, 0, 2, 0, 0, 0, 0],
                            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                            [0, 0, 2, 0, 0, 0, 0, 2, 0, 0],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 2, 0, 0, 0, 0, 0],
                            [0, 2, 0, 0, 0, 0, 0, 2, 0, 0],
                            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                            [0, 0, 0, 2, 0, 2, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 2]
                        ],
                    },
                    LevelInfo {
                        name: "Obsidian Core",
                        difficulty: "EXPERT",
                        biome: "Volcanic Caldera",
                        biome_emoji: "🌋",
                        description: "A hellish volcanic basin filled with pools of bubbling lava. Survive intense waves under extreme heat. Relies on a complex procedural seed.",
                        map_id: "volcanic",
                        unlocked: save.hard.completed,
                        completed: save.hard.completed && save.hard.high_score > 0,
                        high_score: if save.hard.completed { save.hard.high_score } else { 0 },
                        grid: [
                            [2, 2, 0, 0, 0, 0, 0, 0, 2, 2],
                            [2, 0, 0, 1, 1, 1, 1, 0, 0, 2],
                            [0, 0, 1, 1, 0, 0, 1, 1, 0, 0],
                            [0, 1, 1, 0, 2, 2, 0, 1, 1, 0],
                            [1, 1, 0, 0, 2, 2, 0, 0, 1, 1],
                            [1, 1, 0, 0, 2, 2, 0, 0, 1, 1],
                            [0, 1, 1, 0, 2, 2, 0, 1, 1, 0],
                            [0, 0, 1, 1, 0, 0, 1, 1, 0, 0],
                            [2, 0, 0, 1, 1, 1, 1, 0, 0, 2],
                            [2, 2, 0, 0, 0, 0, 0, 0, 2, 2]
                        ],
                    },
                    LevelInfo {
                        name: "Lost Haven",
                        difficulty: "SANDBOX",
                        biome: "Cosmic Nexus",
                        biome_emoji: "🌀",
                        description: "An infinite procedural sector outside normal space-time. The island reshapes itself dynamically on every jump. Build without limits!",
                        map_id: "procedural",
                        unlocked: true,
                        completed: false,
                        high_score: 0,
                        grid: [
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                            [0, 0, 0, 2, 2, 2, 0, 0, 0, 0],
                            [0, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                            [0, 2, 0, 0, 0, 0, 0, 2, 0, 0],
                            [0, 2, 0, 0, 0, 0, 0, 2, 0, 0],
                            [0, 2, 0, 0, 0, 0, 0, 2, 0, 0],
                            [0, 0, 2, 0, 0, 0, 2, 0, 0, 0],
                            [0, 0, 0, 2, 2, 2, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
                        ],
                    },
                ];

                let selected_idx = self.selected_level_point.unwrap_or(0).min(levels.len() - 1);

                let (theme_bg, theme_accent) = match selected_idx {
                    0 => (egui::Color32::from_rgb(10, 35, 15), egui::Color32::from_rgb(0, 230, 115)),
                    1 => (egui::Color32::from_rgb(45, 30, 10), egui::Color32::from_rgb(255, 170, 0)),
                    2 => (egui::Color32::from_rgb(10, 25, 45), egui::Color32::from_rgb(0, 191, 255)),
                    3 => (egui::Color32::from_rgb(35, 10, 10), egui::Color32::from_rgb(255, 64, 64)),
                    _ => (egui::Color32::from_rgb(25, 10, 40), egui::Color32::from_rgb(224, 64, 251)),
                };

                egui::Area::new(egui::Id::new("hud_level_select"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(&egui_ctx, |ui| {
                        egui::Frame::NONE
                            .fill(egui::Color32::from_black_alpha(235))
                            .corner_radius(16.0)
                            .stroke(egui::Stroke::new(2.5, theme_accent))
                            .inner_margin(24.0)
                            .show(ui, |ui| {
                                ui.set_width(920.0);
                                ui.vertical(|ui| {
                                    // Title block
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new("CAMPAIGN SECTOR SELECTOR")
                                                .font(egui::FontId::proportional(28.0))
                                                .color(theme_accent)
                                                .strong()
                                        );
                                        ui.label(
                                            egui::RichText::new("Warp coordinates unlocked. Scroll and select a sector node.")
                                                .font(egui::FontId::proportional(14.0))
                                                .color(egui::Color32::LIGHT_GRAY)
                                        );
                                    });
                                    ui.add_space(20.0);

                                    ui.horizontal(|ui| {
                                        // Left Area: Scrollable Map
                                        ui.allocate_ui(egui::vec2(600.0, 340.0), |ui| {
                                            egui::Frame::canvas(ui.style())
                                                .fill(egui::Color32::from_rgb(15, 15, 18))
                                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
                                                .corner_radius(8.0)
                                                .inner_margin(0.0)
                                                .show(ui, |ui| {
                                                    ui.set_height(340.0);
                                                    // Scroll Area
                                                    egui::ScrollArea::horizontal()
                                                        .id_source("level_scroll")
                                                        .max_height(340.0)
                                                        .show(ui, |ui| {
                                                            let width = 1600.0;
                                                            let height = 320.0;
                                                            let (rect, _response) = ui.allocate_exact_size(
                                                                egui::vec2(width, height),
                                                                egui::Sense::click_and_drag(),
                                                            );
                                                            
                                                            let painter = ui.painter_at(rect);
                                                            
                                                            // Interpolate and paint the background biomes
                                                            let segments = 20;
                                                            let seg_width = width / segments as f32;
                                                            let biome_colors = vec![
                                                                egui::Color32::from_rgb(10, 45, 20),   // Grasslands
                                                                egui::Color32::from_rgb(55, 40, 15),   // Desert
                                                                egui::Color32::from_rgb(15, 30, 60),   // Tundra
                                                                egui::Color32::from_rgb(45, 15, 15),   // Volcanic
                                                                egui::Color32::from_rgb(25, 10, 45),   // Cosmic
                                                            ];
                                                            
                                                            for s in 0..segments {
                                                                let x_min = rect.min.x + s as f32 * seg_width;
                                                                let x_max = x_min + seg_width;
                                                                
                                                                let pos_pct = s as f32 / (segments - 1) as f32;
                                                                let float_idx = pos_pct * (biome_colors.len() - 1) as f32;
                                                                let idx_lower = float_idx.floor() as usize;
                                                                let idx_upper = float_idx.ceil() as usize;
                                                                let t = float_idx - idx_lower as f32;
                                                                
                                                                let col_lower = biome_colors[idx_lower];
                                                                let col_upper = biome_colors[idx_upper];
                                                                
                                                                let col = egui::Color32::from_rgb(
                                                                    (col_lower.r() as f32 + (col_upper.r() as f32 - col_lower.r() as f32) * t) as u8,
                                                                    (col_lower.g() as f32 + (col_upper.g() as f32 - col_lower.g() as f32) * t) as u8,
                                                                    (col_lower.b() as f32 + (col_upper.b() as f32 - col_lower.b() as f32) * t) as u8,
                                                                );
                                                                
                                                                painter.rect_filled(
                                                                    egui::Rect::from_min_max(
                                                                        egui::pos2(x_min, rect.min.y),
                                                                        egui::pos2(x_max, rect.max.y),
                                                                    ),
                                                                    0.0,
                                                                    col,
                                                                );
                                                            }
                                                            
                                                            // Draw subtle grid lines
                                                            for x in (100..1600).step_by(100) {
                                                                painter.line_segment(
                                                                    [
                                                                        egui::pos2(rect.min.x + x as f32, rect.min.y),
                                                                        egui::pos2(rect.min.x + x as f32, rect.max.y)
                                                                    ],
                                                                    egui::Stroke::new(1.0, egui::Color32::from_white_alpha(10)),
                                                                );
                                                            }
                                                            
                                                            // Coordinates for the 5 level points
                                                            let points = vec![
                                                                egui::pos2(rect.min.x + 160.0, rect.min.y + 150.0),
                                                                egui::pos2(rect.min.x + 480.0, rect.min.y + 90.0),
                                                                egui::pos2(rect.min.x + 800.0, rect.min.y + 210.0),
                                                                egui::pos2(rect.min.x + 1120.0, rect.min.y + 100.0),
                                                                egui::pos2(rect.min.x + 1440.0, rect.min.y + 160.0),
                                                            ];
                                                            
                                                            // Draw the curved dotted connecting path using a cubic Bezier curve
                                                            let cubic_bezier = |t: f32, p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2| -> egui::Pos2 {
                                                                let u = 1.0 - t;
                                                                let tt = t * t;
                                                                let uu = u * u;
                                                                let uuu = uu * u;
                                                                let ttt = tt * t;
                                                                egui::pos2(
                                                                    uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
                                                                    uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
                                                                )
                                                            };

                                                            let dot_colors = vec![
                                                                egui::Color32::from_rgb(120, 240, 160), // Grasslands
                                                                egui::Color32::from_rgb(255, 200, 100), // Desert
                                                                egui::Color32::from_rgb(150, 220, 255), // Tundra
                                                                egui::Color32::from_rgb(255, 120, 100), // Volcanic
                                                                egui::Color32::from_rgb(220, 160, 255), // Cosmic
                                                            ];

                                                            for w in 0..(points.len() - 1) {
                                                                let p0 = points[w];
                                                                let p3 = points[w+1];
                                                                let dx = p3.x - p0.x;
                                                                let p1 = p0 + egui::vec2(dx * 0.5, 0.0);
                                                                let p2 = p3 - egui::vec2(dx * 0.5, 0.0);

                                                                let distance = ((p3.x - p0.x).powi(2) + (p3.y - p0.y).powi(2)).sqrt();
                                                                let dot_spacing = 16.0;
                                                                let num_dots = (distance / dot_spacing).round() as usize;
                                                                let num_dots = num_dots.max(2);

                                                                let col_start = dot_colors[w];
                                                                let col_end = dot_colors[w+1];

                                                                for d in 0..=num_dots {
                                                                    let t = d as f32 / num_dots as f32;
                                                                    let pos = cubic_bezier(t, p0, p1, p2, p3);

                                                                    let dot_color = egui::Color32::from_rgb(
                                                                        (col_start.r() as f32 + (col_end.r() as f32 - col_start.r() as f32) * t) as u8,
                                                                        (col_start.g() as f32 + (col_end.g() as f32 - col_start.g() as f32) * t) as u8,
                                                                        (col_start.b() as f32 + (col_end.b() as f32 - col_start.b() as f32) * t) as u8,
                                                                    );

                                                                    // Draw shadow outline circle
                                                                    painter.circle_filled(pos, 4.0, egui::Color32::from_black_alpha(180));
                                                                    // Draw inner colored circle
                                                                    painter.circle_filled(pos, 2.0, dot_color);
                                                                }
                                                            }
                                                            
                                                            // Render the interactive nodes
                                                            for (i, lvl) in levels.iter().enumerate() {
                                                                let pt = points[i];
                                                                
                                                                let node_size = 50.0;
                                                                let node_rect = egui::Rect::from_center_size(pt, egui::vec2(node_size, node_size));
                                                                let node_resp = ui.allocate_rect(node_rect, egui::Sense::click());
                                                                
                                                                if node_resp.clicked() {
                                                                    self.selected_level_point = Some(i);
                                                                    self.audio.play_click();
                                                                }
                                                                
                                                                let is_selected = selected_idx == i;
                                                                let is_hovered = node_resp.hovered();
                                                                
                                                                let (glow_color, node_bg, border_color) = if !lvl.unlocked {
                                                                    (
                                                                        egui::Color32::from_rgb(80, 20, 20),
                                                                        egui::Color32::from_rgb(30, 20, 20),
                                                                        egui::Color32::from_rgb(100, 40, 40)
                                                                    )
                                                                } else if is_selected {
                                                                    (
                                                                        theme_accent,
                                                                        theme_bg,
                                                                        theme_accent
                                                                    )
                                                                } else if is_hovered {
                                                                    (
                                                                        egui::Color32::from_rgb(255, 255, 255),
                                                                        egui::Color32::from_rgb(45, 45, 50),
                                                                        egui::Color32::WHITE
                                                                    )
                                                                } else {
                                                                    (
                                                                        egui::Color32::from_white_alpha(50),
                                                                        egui::Color32::from_rgb(25, 25, 28),
                                                                        egui::Color32::from_rgb(180, 180, 180)
                                                                    )
                                                                };
                                                                
                                                                let glow_radius = if is_selected { 26.0 } else if is_hovered { 22.0 } else { 18.0 };
                                                                painter.circle_filled(
                                                                    pt,
                                                                    glow_radius,
                                                                    egui::Color32::from_rgba_unmultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 60),
                                                                );
                                                                
                                                                painter.circle(
                                                                    pt,
                                                                    16.0,
                                                                    node_bg,
                                                                    egui::Stroke::new(2.0, border_color),
                                                                );
                                                                
                                                                let node_text = if !lvl.unlocked {
                                                                    "🔒".to_string()
                                                                } else {
                                                                    lvl.biome_emoji.to_string()
                                                                };
                                                                
                                                                painter.text(
                                                                    pt + egui::vec2(0.0, 1.0),
                                                                    egui::Align2::CENTER_CENTER,
                                                                    node_text,
                                                                    egui::FontId::proportional(16.0),
                                                                    egui::Color32::WHITE,
                                                                );
                                                                
                                                                let text_offset = if i % 2 == 0 { -32.0 } else { 32.0 };
                                                                let text_color = if is_selected { theme_accent } else if is_hovered { egui::Color32::WHITE } else { egui::Color32::GRAY };
                                                                
                                                                painter.text(
                                                                    pt + egui::vec2(0.0, text_offset),
                                                                    egui::Align2::CENTER_CENTER,
                                                                    format!("Node {}: {}", i + 1, lvl.name),
                                                                    egui::FontId::proportional(12.0),
                                                                    text_color,
                                                                );
                                                            }
                                                        });
                                                });
                                        });
                                        
                                        ui.add_space(16.0);

                                        // Right Area: Details Panel
                                        let selected_level = &levels[selected_idx];
                                        ui.allocate_ui(egui::vec2(280.0, 340.0), |ui| {
                                            egui::Frame::group(ui.style())
                                                .fill(egui::Color32::from_rgb(20, 20, 24))
                                                .stroke(egui::Stroke::new(1.5, theme_accent))
                                                .corner_radius(8.0)
                                                .inner_margin(12.0)
                                                .show(ui, |ui| {
                                                    ui.set_height(316.0);
                                                    ui.set_width(256.0);
                                                    ui.vertical(|ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.label(
                                                                egui::RichText::new(selected_level.biome_emoji)
                                                                    .font(egui::FontId::proportional(22.0))
                                                            );
                                                            ui.vertical(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new(selected_level.name)
                                                                        .font(egui::FontId::proportional(16.0))
                                                                        .color(egui::Color32::WHITE)
                                                                        .strong()
                                                                );
                                                                ui.label(
                                                                    egui::RichText::new(format!("{} BIOME", selected_level.biome.to_uppercase()))
                                                                        .font(egui::FontId::proportional(10.0))
                                                                        .color(theme_accent)
                                                                        .strong()
                                                                );
                                                            });
                                                        });
                                                        
                                                        ui.add_space(8.0);
                                                        ui.separator();
                                                        ui.add_space(8.0);
                                                        
                                                        ui.vertical_centered(|ui| {
                                                            draw_minimap_preview(ui, &selected_level.grid);
                                                        });
                                                        
                                                        ui.add_space(8.0);
                                                        
                                                        ui.horizontal(|ui| {
                                                            ui.label(
                                                                egui::RichText::new("DIFFICULTY:")
                                                                    .font(egui::FontId::proportional(11.0))
                                                                    .color(egui::Color32::GRAY)
                                                            );
                                                            let diff_color = match selected_level.difficulty {
                                                                "EASY" => egui::Color32::from_rgb(102, 187, 106),
                                                                "MEDIUM" => egui::Color32::from_rgb(255, 167, 38),
                                                                "HARD" => egui::Color32::from_rgb(239, 83, 80),
                                                                "EXPERT" => egui::Color32::from_rgb(229, 57, 53),
                                                                _ => egui::Color32::from_rgb(33, 150, 243),
                                                            };
                                                            ui.label(
                                                                egui::RichText::new(selected_level.difficulty)
                                                                    .font(egui::FontId::proportional(11.0))
                                                                    .color(diff_color)
                                                                    .strong()
                                                            );
                                                        });
                                                        
                                                        if selected_level.map_id != "procedural" && selected_level.map_id != "volcanic" {
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new("HIGH SCORE:")
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::GRAY)
                                                                );
                                                                ui.label(
                                                                    egui::RichText::new(format!("{}", selected_level.high_score))
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::from_rgb(255, 215, 0))
                                                                        .strong()
                                                                );
                                                            });
                                                            
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new("STATUS:")
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::GRAY)
                                                                );
                                                                let (status_txt, status_col) = if selected_level.completed {
                                                                    ("COMPLETED", egui::Color32::from_rgb(102, 187, 106))
                                                                } else {
                                                                    ("UNCOMPLETED", egui::Color32::GRAY)
                                                                };
                                                                ui.label(
                                                                    egui::RichText::new(status_txt)
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(status_col)
                                                                        .strong()
                                                                );
                                                            });
                                                        } else if selected_level.map_id == "volcanic" {
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new("HIGH SCORE:")
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::GRAY)
                                                                );
                                                                ui.label(
                                                                    egui::RichText::new(format!("{}", selected_level.high_score))
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::from_rgb(255, 215, 0))
                                                                        .strong()
                                                                );
                                                            });
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new("STATUS:")
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::GRAY)
                                                                );
                                                                let (status_txt, status_col) = if selected_level.completed {
                                                                    ("COMPLETED", egui::Color32::from_rgb(102, 187, 106))
                                                                } else {
                                                                    ("UNCOMPLETED", egui::Color32::GRAY)
                                                                };
                                                                ui.label(
                                                                    egui::RichText::new(status_txt)
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(status_col)
                                                                        .strong()
                                                                );
                                                            });
                                                        } else {
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new("MODE:")
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::GRAY)
                                                                );
                                                                ui.label(
                                                                    egui::RichText::new("FREEBUILD")
                                                                        .font(egui::FontId::proportional(11.0))
                                                                        .color(egui::Color32::from_rgb(33, 150, 243))
                                                                        .strong()
                                                                );
                                                            });
                                                        }
                                                        
                                                        ui.add_space(8.0);
                                                        
                                                        ui.label(
                                                            egui::RichText::new(selected_level.description)
                                                                .font(egui::FontId::proportional(11.0))
                                                                .color(egui::Color32::LIGHT_GRAY)
                                                        );
                                                        
                                                        ui.add_space(14.0);
                                                        
                                                        if !selected_level.unlocked {
                                                            ui.add_enabled_ui(false, |ui| {
                                                                let _ = ui.add_sized(
                                                                    [232.0, 32.0],
                                                                    egui::Button::new(
                                                                        egui::RichText::new("🔒 SECTOR LOCKED")
                                                                            .strong()
                                                                    )
                                                                );
                                                            });
                                                        } else {
                                                            let btn_color = match selected_level.map_id {
                                                                "easy" => egui::Color32::from_rgb(46, 125, 50),
                                                                "medium" => egui::Color32::from_rgb(230, 81, 0),
                                                                "hard" => egui::Color32::from_rgb(198, 40, 40),
                                                                "volcanic" => egui::Color32::from_rgb(183, 28, 28),
                                                                _ => egui::Color32::from_rgb(21, 101, 192),
                                                            };
                                                            
                                                            let play_btn = egui::Button::new(
                                                                egui::RichText::new("🚀 LAUNCH MISSION")
                                                                    .font(egui::FontId::proportional(14.0))
                                                                    .strong()
                                                            ).fill(btn_color);
                                                            
                                                            if ui.add_sized([232.0, 32.0], play_btn).clicked() {
                                                                self.load_level(selected_level.map_id);
                                                                self.show_level_select = false;
                                                                self.audio.play_click();
                                                            }
                                                        }
                                                    });
                                                });
                                        });
                                    });
                                    
                                    ui.add_space(20.0);
                                    
                                    ui.vertical_centered(|ui| {
                                        let back_btn = egui::Button::new(
                                            egui::RichText::new("⬅ Back to Main Menu")
                                                .font(egui::FontId::proportional(14.0))
                                                .strong()
                                        ).fill(egui::Color32::from_rgb(60, 60, 64));
                                        
                                        if ui.add_sized([240.0, 32.0], back_btn).clicked() {
                                            self.show_level_select = false;
                                            self.audio.play_click();
                                        }
                                    });
                                });
                            });
                    });
            } else if self.show_stats_screen {
                // Large modal for Statistics & Achievements
                egui::Area::new(egui::Id::new("hud_stats_screen"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(&egui_ctx, |ui| {
                        egui::Frame::NONE
                            .fill(egui::Color32::from_black_alpha(220)) // Dark frosted glass
                            .corner_radius(16.0)
                            .stroke(egui::Stroke::new(2.5, egui::Color32::from_rgb(230, 81, 0))) // Orange glow
                            .inner_margin(24.0)
                            .show(ui, |ui| {
                                ui.set_width(860.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("COMMANDER STATISTICS")
                                            .font(egui::FontId::proportional(32.0))
                                            .color(egui::Color32::from_rgb(230, 81, 0))
                                            .strong()
                                    );
                                    ui.label(
                                        egui::RichText::new("Tactical performance and operational achievements across campaigns")
                                            .font(egui::FontId::proportional(14.0))
                                            .color(egui::Color32::LIGHT_GRAY)
                                    );
                                    ui.add_space(20.0);

                                    let games = self.statistics.games_played;
                                    let wins = self.statistics.wins;
                                    let waves = self.statistics.waves_completed;
                                    let kills = self.statistics.enemies_killed;
                                    let towers = self.statistics.towers_placed;
                                    let high_score = self.statistics.high_score;

                                    let win_rate = if games > 0 {
                                        (wins as f32 / games as f32) * 100.0
                                    } else {
                                        0.0
                                    };

                                    // Derive Achievements
                                    let achievements = [
                                        ("First Blood", "Eliminate at least 1 enemy", kills >= 1, "🩸"),
                                        ("Tower Cadet", "Construct at least 1 defensive tower", towers >= 1, "🗼"),
                                        ("Novice Commander", "Secure at least 1 campaign victory", wins >= 1, "🎖️"),
                                        ("Wave Rider", "Survive 50 waves in total", waves >= 50, "🌊"),
                                        ("Architect", "Construct 50 defensive towers in total", towers >= 50, "📐"),
                                        ("Slayer", "Eliminate 100 enemies in total", kills >= 100, "⚔️"),
                                    ];

                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(24.0, 0.0);

                                        // Left column: Stats
                                        ui.vertical(|ui| {
                                            ui.set_width(380.0);
                                            egui::Frame::group(ui.style())
                                                .fill(egui::Color32::from_rgb(25, 25, 25))
                                                .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 100, 100)))
                                                .inner_margin(16.0)
                                                .show(ui, |ui| {
                                                    ui.vertical(|ui| {
                                                        ui.label(
                                                            egui::RichText::new("OPERATIONAL DATA")
                                                                .font(egui::FontId::proportional(18.0))
                                                                .color(egui::Color32::WHITE)
                                                                .strong()
                                                        );
                                                        ui.add_space(12.0);

                                                        ui.horizontal(|ui| {
                                                            ui.label(egui::RichText::new("Games Played:").color(egui::Color32::LIGHT_GRAY));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(egui::RichText::new(games.to_string()).color(egui::Color32::WHITE).strong());
                                                            });
                                                        });
                                                        ui.add_space(8.0);
                                                        ui.separator();
                                                        ui.add_space(8.0);

                                                        ui.horizontal(|ui| {
                                                            ui.label(egui::RichText::new("Campaign Wins:").color(egui::Color32::LIGHT_GRAY));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(egui::RichText::new(wins.to_string()).color(egui::Color32::WHITE).strong());
                                                            });
                                                        });
                                                        ui.add_space(8.0);
                                                        ui.separator();
                                                        ui.add_space(8.0);

                                                        ui.horizontal(|ui| {
                                                            ui.label(egui::RichText::new("Win Rate:").color(egui::Color32::LIGHT_GRAY));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(egui::RichText::new(format!("{:.1}%", win_rate)).color(egui::Color32::WHITE).strong());
                                                            });
                                                        });
                                                        ui.add_space(8.0);
                                                        ui.separator();
                                                        ui.add_space(8.0);

                                                        ui.horizontal(|ui| {
                                                            ui.label(egui::RichText::new("Waves Completed:").color(egui::Color32::LIGHT_GRAY));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(egui::RichText::new(waves.to_string()).color(egui::Color32::WHITE).strong());
                                                            });
                                                        });
                                                        ui.add_space(8.0);
                                                        ui.separator();
                                                        ui.add_space(8.0);

                                                        ui.horizontal(|ui| {
                                                            ui.label(egui::RichText::new("Enemies Terminated:").color(egui::Color32::LIGHT_GRAY));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(egui::RichText::new(kills.to_string()).color(egui::Color32::WHITE).strong());
                                                            });
                                                        });
                                                        ui.add_space(8.0);
                                                        ui.separator();
                                                        ui.add_space(8.0);

                                                        ui.horizontal(|ui| {
                                                            ui.label(egui::RichText::new("Towers Placed:").color(egui::Color32::LIGHT_GRAY));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(egui::RichText::new(towers.to_string()).color(egui::Color32::WHITE).strong());
                                                            });
                                                        });
                                                        ui.add_space(8.0);
                                                        ui.separator();
                                                        ui.add_space(8.0);

                                                        ui.horizontal(|ui| {
                                                            ui.label(egui::RichText::new("Personal High Score:").color(egui::Color32::from_rgb(255, 215, 0)));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(egui::RichText::new(high_score.to_string()).color(egui::Color32::from_rgb(255, 215, 0)).strong());
                                                            });
                                                        });
                                                    });
                                                });
                                        });

                                        // Right column: Achievements
                                        ui.vertical(|ui| {
                                            ui.set_width(400.0);
                                            egui::Frame::group(ui.style())
                                                .fill(egui::Color32::from_rgb(25, 25, 25))
                                                .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 100, 100)))
                                                .inner_margin(16.0)
                                                .show(ui, |ui| {
                                                    ui.vertical(|ui| {
                                                        let completed_count = achievements.iter().filter(|&&(_, _, unlocked, _)| unlocked).count();
                                                        let pct = completed_count as f32 / achievements.len() as f32;
                                                        
                                                        ui.horizontal(|ui| {
                                                            ui.label(
                                                                egui::RichText::new("TACTICAL HONORS")
                                                                    .font(egui::FontId::proportional(18.0))
                                                                    .color(egui::Color32::WHITE)
                                                                    .strong()
                                                            );
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.label(
                                                                    egui::RichText::new(format!("{} / {}", completed_count, achievements.len()))
                                                                        .color(egui::Color32::from_rgb(230, 81, 0))
                                                                        .strong()
                                                                );
                                                            });
                                                        });
                                                        ui.add_space(8.0);
                                                        
                                                        // Progress Bar
                                                        ui.add(
                                                            egui::ProgressBar::new(pct)
                                                                .text(format!("{:.0}%", pct * 100.0))
                                                                .fill(egui::Color32::from_rgb(230, 81, 0))
                                                        );
                                                        ui.add_space(16.0);

                                                        // List Achievements
                                                        for (name, desc, unlocked, emoji) in achievements.iter() {
                                                            ui.horizontal(|ui| {
                                                                if *unlocked {
                                                                    ui.label(egui::RichText::new(*emoji).font(egui::FontId::proportional(22.0)));
                                                                    ui.vertical(|ui| {
                                                                        ui.label(egui::RichText::new(*name).color(egui::Color32::from_rgb(255, 215, 0)).strong());
                                                                        ui.label(egui::RichText::new(*desc).color(egui::Color32::LIGHT_GRAY).font(egui::FontId::proportional(11.0)));
                                                                    });
                                                                } else {
                                                                    ui.label(egui::RichText::new("🔒").font(egui::FontId::proportional(22.0)));
                                                                    ui.vertical(|ui| {
                                                                        ui.label(egui::RichText::new(*name).color(egui::Color32::GRAY).strong());
                                                                        ui.label(egui::RichText::new(*desc).color(egui::Color32::DARK_GRAY).font(egui::FontId::proportional(11.0)));
                                                                    });
                                                                }
                                                            });
                                                            ui.add_space(6.0);
                                                        }
                                                    });
                                                });
                                        });
                                    });

                                    ui.add_space(20.0);

                                    // Back Button
                                    let back_btn = egui::Button::new(egui::RichText::new("⬅ Back to Menu").strong())
                                        .fill(egui::Color32::from_rgb(70, 70, 70));
                                    if ui.add_sized([200.0, 32.0], back_btn).clicked() {
                                        self.show_stats_screen = false;
                                        self.audio.play_click();
                                    }
                                });
                            });
                    });
            } else {
                // Centered Main Menu modal
                egui::Area::new(egui::Id::new("hud_main_menu"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(&egui_ctx, |ui| {
                        egui::Frame::NONE
                            .fill(egui::Color32::from_black_alpha(200)) // Deep dark frosted glass
                            .corner_radius(16.0)
                            .stroke(egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 230, 255))) // Glowing cyan border
                            .inner_margin(32.0)
                            .show(ui, |ui| {
                                ui.set_width(320.0);
                                ui.vertical_centered(|ui| {
                                    // Title with custom font size/color
                                    ui.label(
                                        egui::RichText::new("ISOGUARD")
                                            .font(egui::FontId::proportional(48.0))
                                            .color(egui::Color32::from_rgb(0, 230, 255))
                                            .strong()
                                    );
                                    
                                    ui.label(
                                        egui::RichText::new("ISOMETRIC TOWER DEFENSE")
                                            .font(egui::FontId::proportional(12.0))
                                            .color(egui::Color32::from_rgb(255, 215, 0)) // Gold
                                            .strong()
                                    );
                                    
                                    ui.add_space(24.0);
                                    
                                    let btn_width = 240.0;
                                    let btn_height = 40.0;

                                    if !self.show_options_menu {
                                        // 1. Play Button (Opens Level Select)
                                        let play_btn = egui::Button::new(
                                            egui::RichText::new("▶  Start Game")
                                                .font(egui::FontId::proportional(18.0))
                                                .color(egui::Color32::WHITE)
                                                .strong()
                                        )
                                        .fill(egui::Color32::from_rgb(46, 125, 50)) // Emerald green
                                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(102, 187, 106)));

                                        if ui.add_sized([btn_width, btn_height], play_btn).clicked() {
                                            self.show_level_select = true;
                                            self.audio.play_click();
                                        }

                                        ui.add_space(16.0);

                                        // Continue Game Button (loads from save file)
                                        let save_exists = std::path::Path::new("isoguard_save.json").exists();
                                        ui.add_enabled_ui(save_exists, |ui| {
                                            let continue_btn = egui::Button::new(
                                                egui::RichText::new("📂  Continue Game")
                                                    .font(egui::FontId::proportional(18.0))
                                                    .strong()
                                            )
                                            .fill(egui::Color32::from_rgb(74, 20, 140)) // Sleek Purple
                                            .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(186, 104, 200)));

                                            if ui.add_sized([btn_width, btn_height], continue_btn).clicked() {
                                                if let Err(e) = self.load_game() {
                                                    self.save_feedback = Some((format!("Load failed: {}", e), 3.0));
                                                }
                                                self.audio.play_click();
                                            }
                                        });

                                        ui.add_space(16.0);


                                        // 2. Options Button
                                        let options_btn = egui::Button::new(
                                            egui::RichText::new("⚙  Options")
                                                .font(egui::FontId::proportional(18.0))
                                                .color(egui::Color32::WHITE)
                                                .strong()
                                        )
                                        .fill(egui::Color32::from_rgb(21, 101, 192)) // Royal blue
                                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 181, 246)));

                                        if ui.add_sized([btn_width, btn_height], options_btn).clicked() {
                                            self.show_options_menu = true;
                                            self.audio.play_click();
                                        }

                                        ui.add_space(16.0);

                                        // 3. Stats & Achievements Button
                                        let stats_btn = egui::Button::new(
                                            egui::RichText::new("🏆  Stats & Achievements")
                                                .font(egui::FontId::proportional(18.0))
                                                .color(egui::Color32::WHITE)
                                                .strong()
                                        )
                                        .fill(egui::Color32::from_rgb(230, 81, 0)) // Bright orange
                                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 183, 77)));

                                        if ui.add_sized([btn_width, btn_height], stats_btn).clicked() {
                                            self.show_stats_screen = true;
                                             // Reload stats from disk when opening to make sure they are fresh
                                             let save = crate::game::progress::SaveData::load();
                                             self.statistics = save.statistics;
                                            self.audio.play_click();
                                        }

                                        ui.add_space(16.0);

                                        // 3. Quit Button
                                        let quit_btn = egui::Button::new(
                                            egui::RichText::new("🚪  Quit Game")
                                                .font(egui::FontId::proportional(18.0))
                                                .color(egui::Color32::WHITE)
                                                .strong()
                                        )
                                        .fill(egui::Color32::from_rgb(198, 40, 40)) // Crimson red
                                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(239, 83, 80)));

                                        if ui.add_sized([btn_width, btn_height], quit_btn).clicked() {
                                            self.exit_requested = true;
                                            self.audio.play_click();
                                        }
                                    } else {
                                        // OPTIONS SUB-MENU
                                        ui.label(
                                            egui::RichText::new("SETTINGS")
                                                .font(egui::FontId::proportional(20.0))
                                                .color(egui::Color32::WHITE)
                                                .strong()
                                        );
                                        
                                        ui.add_space(16.0);

                                        // SFX Volume Control Slider
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("🔊 SFX Vol:").color(egui::Color32::WHITE).font(egui::FontId::proportional(15.0)));
                                            let mut sfx_vol = self.audio.get_volume();
                                            if ui.add(egui::Slider::new(&mut sfx_vol, 0.0..=1.0)).changed() {
                                                self.audio.set_volume(sfx_vol);
                                            }
                                        });

                                        ui.add_space(12.0);

                                        // Music Volume Control Slider
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("🎵 Music Vol:").color(egui::Color32::WHITE).font(egui::FontId::proportional(15.0)));
                                            let mut music_vol = self.audio.get_music_volume();
                                            if ui.add(egui::Slider::new(&mut music_vol, 0.0..=1.0)).changed() {
                                                self.audio.set_music_volume(music_vol);
                                            }
                                        });

                                        ui.add_space(12.0);

                                        // Music Mute Toggle Checkbox
                                        ui.horizontal(|ui| {
                                            let mut music_muted = self.audio.get_music_muted();
                                            if ui.checkbox(&mut music_muted, egui::RichText::new("Mute Music").color(egui::Color32::WHITE).font(egui::FontId::proportional(15.0))).changed() {
                                                self.audio.set_music_muted(music_muted);
                                                self.audio.play_click();
                                            }
                                        });

                                        ui.add_space(24.0);

                                        // Back Button
                                        let back_btn = egui::Button::new(
                                            egui::RichText::new("⬅  Back")
                                                .font(egui::FontId::proportional(18.0))
                                                .color(egui::Color32::WHITE)
                                                .strong()
                                        )
                                        .fill(egui::Color32::from_rgb(97, 97, 97))
                                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(189, 189, 189)));

                                        if ui.add_sized([btn_width, btn_height], back_btn).clicked() {
                                            self.show_options_menu = false;
                                            self.audio.play_click();
                                        }
                                    }
                                });
                            });
                    });
            }
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

                if (self.game_state == crate::game::game_state::GameState::Playing || self.game_state == crate::game::game_state::GameState::Paused)
                    && *button == winit::event::MouseButton::Left
                    && *state == winit::event::ElementState::Pressed
                {
                    if let Some((gx, gz)) = self.hovered_tile {
                        // Check if a tower is at the clicked tile
                        let mut clicked_tower_idx = None;
                        for (idx, tower) in self.tower_manager.towers.iter().enumerate() {
                            let t_pos = tower.get_position();
                            if t_pos.x.round() as i32 == gx && t_pos.y.round() as i32 == gz {
                                clicked_tower_idx = Some(idx);
                                break;
                            }
                        }

                        if let Some(idx) = clicked_tower_idx {
                            self.selected_tower_index = Some(idx);
                            self.placement_status = format!("Selected level {} {:?}", self.tower_manager.towers[idx].get_level(), self.tower_manager.towers[idx].tower_type());
                            self.update_window_title();
                        } else if self.selected_tower_index.is_some() {
                            self.selected_tower_index = None;
                            self.placement_status = "Ready".to_string();
                            self.update_window_title();
                        } else {
                            let pos = glam::Vec2::new(gx as f32, gz as f32);
                            let tower_type = self.selected_tower_type;
                            match self.tower_manager.place_tower(&self.map, pos, tower_type, &mut self.economy) {
                                Ok(_) => {
                                    self.audio.play_place();
                                    log::info!("Placed {:?} tower at {}, {}", tower_type, gx, gz);
                                    self.placement_status = format!("Placed {:?} tower at ({}, {})", tower_type, gx, gz);
                                    self.update_window_title();

                                    self.statistics.towers_placed += 1;
                                    self.save_statistics();
                                }
                                Err(e) => {
                                    log::warn!("Failed to place tower: {}", e);
                                    self.placement_status = format!("Failed to place tower: {}", e);
                                    self.update_window_title();
                                }
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

fn draw_minimap_preview(ui: &mut egui::Ui, grid: &[[u32; 10]; 10]) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
        for row in grid {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                for &cell in row {
                    let color = match cell {
                        1 => egui::Color32::from_rgb(218, 165, 32), // Path (gold)
                        2 => egui::Color32::from_rgb(120, 120, 120), // Rock (grey)
                        _ => egui::Color32::from_rgb(34, 139, 34), // Grass (forest green)
                    };
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 1.0, color);
                }
            });
        }
    });
}
