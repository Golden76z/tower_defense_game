# window.rs Decomposition — Design Spec

**Date:** 2026-07-23
**Status:** Design approved; pending implementation plan
**Branch:** dev

## Problem

`src/engine/window.rs` is a ~4,235-line god-file. Its `State` struct has ~50 fields
spanning GPU, game simulation, input, audio, and UI, and its methods are enormous:

- `render()` ≈ 2,000 lines — GPU submission **and** all egui **and** direct
  game-state mutation from UI callbacks.
- `prepare_frame()` ≈ 558 lines — per-entity batching with duplicated grid→world math.
- `fixed_update()` ≈ 291 lines — the simulation step.

Consequences:

- Simulation logic is entangled with wgpu, so almost none of it is unit-testable
  (the file has only `ray_aabb` math tests).
- The file is hard to navigate, review, and change; it is the #1 structural risk
  in the codebase and blocks downstream work (perf, data-driven design, new UI).

## Goal

Decompose `State` into focused, well-bounded units with clear interfaces,
**without changing observable game behavior**, in small, individually-verifiable
steps. The primary payoff is a **GPU-free, unit-testable `World`** and a
`render()` that only renders.

Non-goals: any behavior/gameplay change, performance work, remaining Tier-2 bug
fixes, wasm repair, or data-driven enemies/towers. Each is separate work.

## Target architecture

Four homes carved out of `window.rs`:

### `game::world::World` (new, GPU-free)

Owns the simulation and nothing that needs wgpu:
`map`, `enemy_manager`, `tower_manager`, `selected_tower_type`, `projectiles`,
`particles`, `economy`, `player_stats`, `wave_manager`, `spatial_grid`,
`query_scratch`, `game_state`, `continued_after_victory`, `enemy_spawn_count`,
`popups`, `time`, `time_elapsed`, `statistics`, `current_map_name`.

API (moved off `State`): `update`/`fixed_update`, spawn & step helpers,
place/upgrade tower, `load_level(map)`, save/restore snapshot. All pure sim.

**Payoff:** the simulation becomes unit-testable. Characterization tests cover
`update`/spawn/economy/collision as each moves in.

### `engine::gfx::GfxContext` (new)

Owns all wgpu: `surface`, `device`, `queue`, `config`, `is_surface_configured`,
`renderer`, the four batchers (`batcher`, `ui_batcher`, `overlay_batcher`,
`terrain_batcher`), the models (`enemy`/`fast_enemy`/`tower`/`sniper`/`projectile`),
texture ids (`highlight`, `popup`, enemy/fast-enemy), `wave_ui_texture`(+id,
`last_wave_ui_text`), egui (`egui_ctx`/`egui_state`/`egui_renderer`), and clear `color`.

API: `resize`, `begin_frame`/`submit`, `register_texture`, batcher access, egui run/paint.

### `engine::ui::*` (new module tree)

Per-screen egui as functions that **return actions instead of mutating game state**:
`ui::hud`, `ui::shop`, `ui::main_menu`, `ui::level_select`, `ui::stats`,
`ui::settings`, `ui::wave_banner`.

Shape: `fn(ctx: &egui::Context, view: &WorldView, ui: &UiState) -> Vec<UiAction>`.

`UiAction` enumerates every mutation the panels currently perform inline:
`StartGame`, `LoadLevel(String)`, `SaveGame`, `LoadGame`, `SelectTower(TowerType)`,
`UpgradeTower(usize)`, `SetSpeed(f32)`, `Toggle*` flags, `RequestExit`, … `State`
applies the returned actions against `World`/`UiState` after running the UI.

### `engine::window::State` (slimmed coordinator)

Owns `GfxContext` + `World` + `UiState`. Wires winit events (`input`, `resize`,
event dispatch) and orchestrates the frame:
`world.update()` → run UI (collect `UiAction`s) → apply actions →
`gfx` renders the scene and paints the UI. Drops from ~4,200 lines to a thin shell.

## Field → module map

| Field(s) | Home |
|---|---|
| surface, device, queue, config, is_surface_configured, renderer | GfxContext |
| batcher, ui_batcher, overlay_batcher, terrain_batcher | GfxContext |
| enemy_model, fast_enemy_model, tower_model, sniper_tower_model, projectile_model | GfxContext |
| _enemy_texture_id, _fast_enemy_texture_id, highlight_texture_id, popup_texture_id | GfxContext |
| wave_ui_texture, wave_ui_texture_id, last_wave_ui_text | GfxContext |
| egui_ctx, egui_state, egui_renderer, color | GfxContext |
| map, enemy_manager, tower_manager, selected_tower_type | World |
| projectiles, particles, popups | World |
| economy, player_stats, wave_manager, spatial_grid, query_scratch | World |
| game_state, continued_after_victory, enemy_spawn_count | World |
| time, time_elapsed, statistics, current_map_name | World |
| cursor_pos, hovered_tile, input | UiState (input) |
| show_path_debug, placement_status, selected_tower_index | UiState |
| show_options_menu, show_level_select, selected_level_point | UiState |
| show_stats_screen, show_ingame_settings, store_open, store_animation | UiState |
| save_feedback, is_loading_save, exit_requested | UiState |
| window, camera, camera_controller | State (shared: events + gfx + input) |

## Migration strategy: behavior-first, incremental

Nearly every method touches fields across concerns, so regrouping fields first
triggers a massive borrow-checker fight that is hard to keep compiling. Instead:
extract method *bodies* into modules that take explicit borrows (shrinking the
methods first), and regroup fields into sub-structs **last**, once methods are
small and their field usage is explicit.

**Invariant for every step:** the tree compiles; `cargo fmt --check` clean;
`clippy -D warnings` clean; all tests green; committed on its own.

### Steps

1. **Warm-up.** Move file-scope free fns (`build_terrain_mesh`,
   `create_gold_texture`, `create_multiline_text_texture`, `draw_minimap_preview`,
   `ray_aabb`) into `engine::render_helpers`. Pure move, ~400 lines out.
2. **Extract `World`.** Create `game::world::World`; move sim fields +
   `fixed_update`/spawn/step logic onto it; `State` holds a `World` and delegates.
   Add characterization tests for `World::update`.
3. **Extract UI.** Move egui panels into `engine::ui::*` with the `UiAction`
   pattern; `render()` runs UI → applies actions. Removes ~1,800 lines.
4. **Extract `GfxContext`.** Group GPU fields into a struct with a small API.
5. **Group `UiState`.** Remaining flags/toggles into one struct; `State` is thin.

Each step is one (or a few) commits and can stop cleanly — the codebase is always
shippable between steps.

## Verification (this file has no behavioral tests)

- Type-system + green gates at every step: compile, `fmt --check`, `clippy -D warnings`, tests.
- Characterization tests for every extracted pure/sim chunk (as done for
  `closest_target_within`).
- User smoke-tests the running game at milestones (after steps 2, 3, 4): a live
  GPU is required, which the agent cannot drive.
- Strictly behavior-preserving: move code, do not rewrite it.

## Risks & mitigations

- **Borrow checker** — behavior-first ordering keeps fields in place until
  methods are small.
- **egui closures mutate `self`** — the `UiAction` indirection is the one place
  code shape changes; keep applied semantics equivalent, extract one panel at a
  time, verify after each.
- **`#[cfg(target_arch = "wasm32")]` blocks** — move intact. wasm already does not
  compile (separate deferred issue), so native is the correctness bar.
- **Scope creep** — restructuring only; no behavior/perf/feature changes.
