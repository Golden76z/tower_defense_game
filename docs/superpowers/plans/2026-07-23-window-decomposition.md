# window.rs Decomposition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the ~4,235-line `src/engine/window.rs` god-file into focused units
(`World`, `GfxContext`, `ui::*`, a slim `State`) without changing observable game
behavior.

**Architecture:** Behavior-first, incremental. Extract method *bodies* into new
modules that take explicit borrows (shrinking methods first); regroup `State`'s
~50 fields into sub-structs last. Every task is a behavior-preserving move that
keeps the tree green and is its own commit.

**Tech Stack:** Rust 2021, wgpu 27, winit 0.30, egui/egui-wgpu/egui-winit 0.33, glam 0.28.

## Global Constraints

- **Behavior-preserving:** no observable gameplay/UX change. Move code; do not rewrite it.
- **Every task ends green**, verified by all four:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo build`
  - `cargo test --all-features`
- **rustfmt bug workaround:** before `cargo fmt` touches `window.rs`, run
  `sed -i 's/[[:space:]]*$//' src/engine/window.rs` (rustfmt 1.9.0 leaves trailing
  whitespace on deeply-nested blank lines and aborts otherwise).
- **One commit per task.** Message prefix `refactor:`.
- **Native is the correctness bar.** Keep every `#[cfg(target_arch = "wasm32")]` block
  intact and in place; do not attempt to make wasm compile here.
- `crate::engine::window::State` must stay a valid path for `src/engine/app.rs`
  (it constructs `State::new` and calls `.input/.render/.update/.resize`), or update
  `app.rs` in the same task.
- For "move existing code" tasks: the code already exists in the repo at the cited
  line range — relocate it verbatim, then adjust only imports/borrows/`self.` paths.
  The plan gives the transformation, not a re-transcription of hundreds of lines.

---

## File Structure

- **Create** `src/engine/render_helpers.rs` — file-scope rendering helpers moved out of `window.rs`.
- **Create** `src/game/world.rs` — `World`: owns the simulation, GPU-free, testable.
- **Create** `src/engine/gfx.rs` — `GfxContext`: owns all wgpu state.
- **Create** `src/engine/ui/mod.rs`, `ui/action.rs`, `ui/view.rs`, and one file per screen
  (`hud.rs`, `shop.rs`, `main_menu.rs`, `level_select.rs`, `stats.rs`, `settings.rs`, `wave_banner.rs`).
- **Modify** `src/engine/mod.rs` — add `pub mod render_helpers; pub mod gfx; pub mod ui;`
- **Modify** `src/game/mod.rs` — add `pub mod world;`
- **Modify** `src/engine/window.rs` — shrinks each task; ends as a thin coordinator.

---

## Phase 1 — Warm-up: extract file-scope free functions

These items are already free functions at file scope (not methods on `State`), so
moving them is a near-pure relocation. Do them first to establish the module wiring
and get an easy green baseline.

Source items (current `window.rs`): `ray_aabb` (~3844), `build_terrain_mesh` (~3870),
`create_gold_texture` (~3944), `create_multiline_text_texture` (~4027),
`draw_minimap_preview` (~4174), and the `#[cfg(test)] mod tests` for `ray_aabb` (~4195).

### Task 1.1: Create `render_helpers` module with `ray_aabb` + its tests

**Files:**
- Create: `src/engine/render_helpers.rs`
- Modify: `src/engine/mod.rs` (add `pub mod render_helpers;`)
- Modify: `src/engine/window.rs` (remove `ray_aabb` + its test module; import from new module)

**Interfaces:**
- Produces: `pub fn ray_aabb(origin: glam::Vec3, dir: glam::Vec3, min: glam::Vec3, max: glam::Vec3) -> Option<f32>`

- [ ] **Step 1** — Create `src/engine/render_helpers.rs`. Move the `ray_aabb` fn body
  verbatim from `window.rs`, adding `pub`. Move the three `ray_aabb_*` tests into a
  `#[cfg(test)] mod tests { use super::*; ... }` in the new file.
- [ ] **Step 2** — In `src/engine/mod.rs`, add `pub mod render_helpers;`.
- [ ] **Step 3** — In `window.rs`, delete the `ray_aabb` fn and the `#[cfg(test)] mod tests`
  that only tests it. Replace call sites (in `pick_tile`) with
  `crate::engine::render_helpers::ray_aabb(...)` (or add `use` at top).
- [ ] **Step 4** — Verify: `cargo test --all-features 2>&1 | grep ray_aabb` shows the 3
  tests now under `engine::render_helpers::tests`, all passing.
- [ ] **Step 5** — Green gates (all four commands above), with the `sed` fmt workaround.
- [ ] **Step 6** — Commit: `refactor: move ray_aabb into engine::render_helpers`

### Task 1.2: Move `build_terrain_mesh` into `render_helpers`

**Interfaces:**
- Consumes: current signature (copy verbatim from `window.rs:~3870`). It returns the
  batched terrain mesh data; keep the exact signature and body.
- Produces: `pub fn build_terrain_mesh(...) -> ...` (same signature, now `pub`).

- [ ] **Step 1** — Cut `build_terrain_mesh` from `window.rs` into `render_helpers.rs`, add `pub`.
- [ ] **Step 2** — Update its call site in `State::rebuild_terrain` to the new path.
- [ ] **Step 3** — Green gates.
- [ ] **Step 4** — Commit: `refactor: move build_terrain_mesh into engine::render_helpers`

### Task 1.3: Move `create_gold_texture` + `create_multiline_text_texture`

- [ ] **Step 1** — Cut both fns into `render_helpers.rs`, add `pub`. They also pull in the
  private char-bitmap helper (`get_char_bitmap`) they call — move that too.
- [ ] **Step 2** — Update call sites in `window.rs` (`prepare_frame` / popup + wave-UI texture
  creation) to the new path.
- [ ] **Step 3** — Green gates. Commit: `refactor: move text-texture builders into engine::render_helpers`

### Task 1.4: Move `draw_minimap_preview`

- [ ] **Step 1** — Cut `draw_minimap_preview` into `render_helpers.rs`, add `pub`.
- [ ] **Step 2** — Update its call site (level-select UI in `render`) to the new path.
- [ ] **Step 3** — Green gates. Commit: `refactor: move draw_minimap_preview into engine::render_helpers`

**Phase 1 milestone:** `window.rs` drops ~400 lines; new `render_helpers.rs` owns the
stateless rendering utilities. No behavior change; no smoke-test needed (pure moves).

---

## Phase 2 — Extract `World` (the high-value phase)

Introduce `game::world::World`, owning the simulation, and move the sim step logic
onto it. `State` gains a `world: World` field and delegates. This makes the sim
unit-testable — the payoff of the whole effort.

### Task 2.1: Create `World` struct owning the simulation fields

**Files:**
- Create: `src/game/world.rs`
- Modify: `src/game/mod.rs` (add `pub mod world;`)
- Modify: `src/engine/window.rs` (`State` loses the sim fields, gains `world: World`)

**Interfaces:**
- Produces:
  ```rust
  pub struct World {
      pub map: crate::game::map::Map,
      pub enemy_manager: crate::game::enemies::EnemyManager,
      pub tower_manager: crate::game::towers::manager::TowerManager,
      pub selected_tower_type: crate::game::towers::manager::TowerType,
      pub projectiles: Vec<crate::game::projectiles::Projectile>,
      pub particles: crate::game::particles::ParticleSystem,
      pub economy: crate::game::economy::Economy,
      pub player_stats: crate::game::player_stats::PlayerStats,
      pub wave_manager: crate::game::wave_manager::WaveManager,
      pub spatial_grid: crate::game::spatial_grid::SpatialGrid,
      pub query_scratch: Vec<usize>,
      pub game_state: crate::game::game_state::GameState,
      pub continued_after_victory: bool,
      pub enemy_spawn_count: usize,
      pub popups: Vec<crate::engine::window::GoldPopup>, // GoldPopup moves to world.rs in this task
      pub statistics: crate::game::progress::GameStatistics,
      pub current_map_name: String,
      pub time_elapsed: f32,
  }
  impl World { pub fn new() -> Self { /* mirror State::new's sim-field initialisers */ } }
  impl Default for World { fn default() -> Self { Self::new() } }
  ```
  (`GoldPopup` moves from `window.rs` into `world.rs` and is re-exported or referenced by
  its new path; update the one construction site in the projectile/reward code.)

- [ ] **Step 1** — Create `src/game/world.rs` with the struct above and `new()`. Copy each
  field's initial value from the corresponding line in `State::new` (`window.rs:~129-430`).
- [ ] **Step 2** — Add `pub mod world;` to `src/game/mod.rs`. Move `GoldPopup` from
  `window.rs` to `world.rs` (`pub use` it from `window.rs` if any external ref needs it).
- [ ] **Step 3** — In `State`, delete the sim fields; add `pub world: crate::game::world::World`.
  In `State::new`, replace the sim-field initialisers with `world: World::new()` (adapting the
  few that need runtime data, e.g. map load — keep that in `State::new`, assign into `world`).
- [ ] **Step 4** — Mechanically update every `self.<simfield>` in `window.rs` to
  `self.world.<simfield>`. (grep `self\.(map|enemy_manager|tower_manager|projectiles|particles|economy|player_stats|wave_manager|spatial_grid|query_scratch|game_state|continued_after_victory|enemy_spawn_count|popups|statistics|current_map_name|time_elapsed|selected_tower_type)\b` and prefix with `world.`.) Note `tower_manager`, `economy`, `player_stats`, `wave_manager`, `game_state`, `selected_tower_index`(stays UI), etc. Public fields accessed from `app.rs`? Check `game_state`, `exit_requested` (exit_requested stays on State).
- [ ] **Step 5** — Green gates. This task has no new tests (pure field relocation); the
  existing 138 tests must all still pass.
- [ ] **Step 6** — Commit: `refactor: introduce game::world::World holding the simulation`

### Task 2.2: Move `fixed_update` onto `World` with characterization tests

**Files:**
- Modify: `src/game/world.rs` (add `World::fixed_update`)
- Modify: `src/engine/window.rs` (`State::fixed_update` becomes `self.world.fixed_update(dt, &mut sink)`)
- Test: `src/game/world.rs` `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `impl World { pub fn fixed_update(&mut self, dt: f32, fx: &mut dyn WorldEvents) }`
  where `WorldEvents` is a trait the caller implements to receive side effects that are
  NOT pure sim (audio cues, particle-at-world-position, gold popups). This keeps `World`
  free of audio/gfx while preserving behavior.
  ```rust
  pub trait WorldEvents {
      fn on_enemy_killed(&mut self, world_pos: glam::Vec3, reward: u32);
      fn on_projectile_fired(&mut self, muzzle: glam::Vec3, dir: glam::Vec2);
      fn on_enemy_leaked(&mut self, count: u32);
      // ...one method per current side-effect in fixed_update
  }
  ```
  `State` implements `WorldEvents` (it has `audio`, `particles` live in world but
  audio + popup-texture live on State/Gfx). Particles stay in `World`; audio calls
  route through `WorldEvents`.

- [ ] **Step 1 (test, RED)** — In `world.rs` tests, write a characterization test that
  builds a `World` with a known map+path, starts wave 0, steps `fixed_update` with a
  no-op `WorldEvents`, and asserts an enemy spawns and advances (position changes toward
  the first waypoint). Use a small `struct NoEvents; impl WorldEvents for NoEvents {...}`.
- [ ] **Step 2** — Run it: `cargo test --all-features -- world::tests` → FAIL (`fixed_update` not on World).
- [ ] **Step 3 (GREEN)** — Move the body of `State::fixed_update` (`window.rs:~756-1047`)
  into `World::fixed_update`, replacing `self.audio.*`/reward-popup/muzzle side effects with
  `fx.on_*` calls, and `self.<field>` with `self.<field>` (now World's own fields). Make
  `State::fixed_update` a thin wrapper: build the event sink and call `self.world.fixed_update(dt, &mut sink)`.
  The sink translates `on_enemy_killed(pos,reward)` → `self.audio.play_explosion()` +
  push GoldPopup + particle burst, exactly as the current code does inline.
- [ ] **Step 4** — Run tests → PASS. Green gates.
- [ ] **Step 5** — Commit: `refactor: move fixed_update onto World behind a WorldEvents sink`
- [ ] **Step 6 (more characterization tests)** — Add tests for: economy gains reward on kill;
  wave transitions after all spawned+cleared; projectile single-target damage via world step.
  Each: RED → GREEN (already green, they document behavior) → commit.

### Task 2.3: Move remaining sim helpers onto `World`

Move `start_game` (sim-reset parts), `load_level`'s world-mutation, and the sim portions
of `update` onto `World` (`World::update(dt, &mut sink)` calling `fixed_update` at fixed
timestep, mirroring current `State::update`). `State::update` keeps only frame-timing +
`prepare_frame` + input glue. Verify + commit per method.

**Phase 2 milestone:** simulation lives in `World`, is GPU-free, and has characterization
tests. **Smoke-test:** run the game — play a level, place/upgrade towers, kill enemies,
win/lose — confirm identical behavior.

---

## Phase 3 — Extract UI into `engine::ui::*` (UiAction pattern)

`render()`'s ~1,800 lines of egui become per-screen functions that read a `WorldView`
and return `Vec<UiAction>`; `render()` applies the actions. Extract one screen per task.

### Task 3.1: Define `UiAction` and `WorldView`

**Files:** Create `src/engine/ui/mod.rs`, `src/engine/ui/action.rs`, `src/engine/ui/view.rs`;
add `pub mod ui;` to `engine::mod.rs`.

**Interfaces:**
- `ui::action::UiAction` — one variant per mutation currently performed inline in egui
  callbacks. Enumerate by grepping `render()` for `self.` writes inside egui closures.
  Known set (extend as found): `StartGame`, `LoadLevel(String)`, `SaveGame`, `LoadGame`,
  `SelectTowerType(TowerType)`, `UpgradeSelectedTower`, `SetSpeedMultiplier(f32)`,
  `ToggleStore`, `ToggleIngameSettings`, `ToggleOptionsMenu`, `ShowLevelSelect(bool)`,
  `ShowStats(bool)`, `SelectLevelPoint(usize)`, `SetMusicVolume(f32)`, `ToggleMute`,
  `RequestExit`, `ContinueAfterVictory`.
- `ui::view::WorldView<'a>` — a read-only projection of the fields the panels display
  (money, lives, wave number/status, selected tower + its upgrade cost, level list +
  save progress, stats, etc.). Borrow, don't clone.
- `pub fn apply(action: UiAction, state: &mut State)` on `State` — the single place
  actions are applied (reuses existing methods: `start_game`, `load_level`, `save_game`…).

- [ ] Define the enum + `WorldView` + an empty `apply` match with a `todo!()`-free arm per
  variant that calls the existing `State` method. Wire `render()` to build a `Vec<UiAction>`,
  currently empty, and apply it (no-op). Green gates. Commit.

### Task 3.2 … 3.8: Extract one screen per task

For each of `hud`, `wave_banner`, `shop`, `settings` (in-game), `main_menu`,
`level_select`, `stats`:

- [ ] Create `src/engine/ui/<screen>.rs` with
  `pub fn <screen>(ctx: &egui::Context, view: &WorldView, ui: &UiState, out: &mut Vec<UiAction>)`.
- [ ] Move that screen's egui block from `render()` into the fn verbatim; replace each
  inline `self.<mutation>` with `out.push(UiAction::…)`; replace each inline `self.<read>`
  with `view.<field>`.
- [ ] In `render()`, call the fn (collect into the actions vec).
- [ ] Green gates + **smoke-test that screen** (it's a UX change in structure). Commit
  `refactor: extract <screen> UI into engine::ui`.

**Phase 3 milestone:** `render()` shrinks to scene submission + `run ui → apply actions →
paint`. Full smoke-test of every screen.

---

## Phase 4 — Extract `GfxContext`

### Task 4.1: Create `GfxContext` and move GPU fields

**Files:** Create `src/engine/gfx.rs`; modify `window.rs`.

**Interfaces:**
- `pub struct GfxContext { surface, device, queue, config, is_surface_configured, renderer,
  batcher, ui_batcher, overlay_batcher, terrain_batcher, models…, texture ids…, wave_ui_*,
  egui_ctx, egui_state, egui_renderer, color }`
- `impl GfxContext { pub async fn new(window) -> Result<Self>; pub fn resize(&mut self, w, h);
  pub fn register_texture(&mut self, tex) -> usize; /* batcher + egui accessors */ }`

- [ ] Move the GPU fields off `State` into `GfxContext`; `State` gains `gfx: GfxContext`.
  Move the wgpu/egui parts of `State::new` into `GfxContext::new`. Update all `self.<gpu>`
  → `self.gfx.<gpu>`. Green gates + smoke-test. Commit.
- [ ] Move `resize` and the render-submission portions of `render()` onto `GfxContext`
  (`gfx.render_scene(&batchers)`, `gfx.paint_egui(...)`). Green gates + smoke-test. Commit.

---

## Phase 5 — Group `UiState`; finalize thin `State`

### Task 5.1: Introduce `UiState`

- [ ] Create `struct UiState { cursor_pos, hovered_tile, input, show_path_debug,
  placement_status, selected_tower_index, show_options_menu, show_level_select,
  selected_level_point, show_stats_screen, show_ingame_settings, store_open,
  store_animation, save_feedback, is_loading_save, exit_requested }` (in `ui/mod.rs` or
  `window.rs`). Move fields off `State`; `State` gains `ui: UiState`. Update accessors.
  `app.rs` reads `state.exit_requested` → expose via `State` accessor or move the check.
- [ ] Green gates + smoke-test. Commit.

**Final milestone:** `State` owns exactly `{ window, camera, camera_controller, gfx, world,
ui, time }` and wires events. `window.rs` is a few hundred lines. Full smoke-test.

---

## Self-review notes

- **Spec coverage:** all four target modules (World, GfxContext, ui::*, slim State) and the
  field→module map from the spec are covered by Phases 2/4/3/5 respectively; Phase 1 covers
  the free-fn helpers.
- **Verification:** every task lists the four green gates; smoke-test milestones are at the
  end of Phases 2, 3, 4, 5 (matching the spec's "after steps 2,3,4" plus final).
- **Type consistency:** `WorldEvents` (2.2), `UiAction`/`WorldView` (3.1), and `GfxContext`
  API (4.1) are the three new interfaces; their names are used consistently across tasks.
- **Known refinement:** Phase 3 task-level egui blocks and Phase 4/5 field lists are
  precise at the module/interface level; the exact per-screen `self.` → `view`/`out`
  substitutions are enumerated during each task by reading the current `render()` (the
  code being moved already exists in-repo). This is relocation, not new code.
