# Developer Memo: Roadmap to Isometric Grid & Visual Map Editor

This memo outlines the step-by-step technical implementation path for migrating from a single hardcoded 3D triangle to a dynamic, instanced isometric 3D grid, followed by screen-to-world mouse picking, map serialization, and ultimately a visual `egui` level editor.

---

## Phase 1: Dynamic 3D Grid Rendering
**Goal:** Replace the hardcoded `VERTICES` array with a dynamic grid of 3D cubes/quads on the X-Z ground plane.
**Addresses Issue:** #29 (*Display tile map with isometric projection*)

### Technical Steps:
1. **Define Grid Cell Dimensions:**
   * Ground plane lies at $Y = 0$.
   * Each grid cell is represented by a 3D unit cube (or a flat quad) of dimensions $1.0 \times 1.0$ (or customized scale).
2. **Implement Instanced Rendering in WGPU:**
   * To prevent duplicate vertex overhead, define a single 3D cube mesh (8 vertices, 36 indices).
   * Define an `InstanceRaw` struct containing a translation matrix (or position offset) and a texture/color index:
     ```rust
     #[repr(C)]
     #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
     pub struct InstanceRaw {
         pub model: [[f32; 4]; 4],
         pub color: [f32; 4],
     }
     ```
   * Update `Renderer` in `src/renderer/mod.rs` to hold an instance buffer. Write to this buffer dynamically when the map changes using `queue.write_buffer`.
3. **Render Pass Update:**
   * In `Renderer::render`, bind the vertex buffer (slot 0), the instance buffer (slot 1), and call `draw_indexed(0..num_indices, 0, 0..num_instances)`.

---

## Phase 2: Screen-to-World Mouse Picking
**Goal:** Select and highlight the grid cell currently under the mouse cursor.
**Addresses Issue:** #30 (*Highlight tile under mouse cursor*)

### Technical Steps:
1. **Construct Cursor Ray:**
   * Retrieve cursor position from `WindowEvent::CursorMoved` normalized to $[-1.0, 1.0]$ clip space.
   * Multiply clip space coordinates by the inverse view-projection matrix ($(\text{Proj} \times \text{View})^{-1}$) to calculate a ray direction in world space.
2. **Ray-Plane Intersection:**
   * Find the intersection point where the ray crosses the ground plane ($Y = 0$):
     $$t = \frac{-ray\_origin.y}{ray\_direction.y}$$
     $$\text{IntersectionPoint} = ray\_origin + t \times ray\_direction$$
3. **Convert to Grid Indices:**
   * Map the intersection coordinate $(X, Z)$ to discrete grid indices:
     $$\text{grid\_x} = X.floor() \text{ as i32}$$
     $$\text{grid\_z} = Z.floor() \text{ as i32}$$
4. **Hover Overlay:**
   * Draw a transparent wireframe cube or highlighted quad at the hovered cell coordinate.

---

## Phase 3: Map Data Model & Serialization
**Goal:** Load and save maps to disk.
**Addresses Issue:** #28 (*Load maps from JSON files*)

### Technical Steps:
1. **Implement Map Schema (`src/game/map/mod.rs`):**
   * Structure maps using `serde`:
     ```rust
     use serde::{Serialize, Deserialize};

     #[derive(Serialize, Deserialize, Clone, Debug)]
     pub struct GameMap {
         pub width: u32,
         pub height: u32,
         pub tiles: Vec<TileType>,
         pub spawn_points: Vec<(u32, u32)>,
         pub base_location: (u32, u32),
         pub path: Vec<(u32, u32)>,
     }

     #[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
     pub enum TileType {
         Empty,
         Path,
         TowerSlot,
         Obstacle,
     }
     ```
2. **Implement File I/O:**
   * Implement helper functions to load/save JSON files from `assets/maps/` directory.

---

## Phase 4: Path & Waypoint Design
**Goal:** Define and visualize enemy paths.
**Addresses Issue:** #33 (*Implement enemy path with waypoints*)

### Technical Steps:
1. **Waypoint Ordering:**
   * Store paths as an ordered list of cell coordinates: `Vec<(u32, u32)>`.
2. **Visual Indicators:**
   * Render indicators at each waypoint (e.g., spawn portals, base banners, or colored flags).
   * Draw lines or arrows connecting waypoints in order to show the movement route.

---

## Phase 5: Visual Editor Integration
**Goal:** Build the GUI tool to edit levels visually using `egui`.
**Addresses Issue:** #66 (*Add multiple maps/levels*)

### Technical Steps:
1. **Uncomment Dependencies:**
   * Uncomment and update `egui`, `egui-wgpu`, and `egui-winit` in `Cargo.toml`.
2. **State & Event Loop Hooks (`src/engine/window.rs` & `src/engine/app.rs`):**
   * Add `egui_ctx`, `egui_state`, and `egui_renderer` into the `State` struct.
   * Hook `egui_state.on_window_event` inside the main `ApplicationHandler::window_event`.
   * Intercept clicks: If `egui_state.wants_pointer_input()` is true, prevent the raycaster from modifying tiles underneath the UI window.
3. **Editor Controls Panel:**
   * Render a panel using `egui::SidePanel` or `egui::Window`:
     * Brush modes: Place/Remove Tile, Add Spawn Point, Place Base, Place Waypoint.
     * Tile selector: Empty, Path, TowerSlot, Obstacle.
     * File controls: Text field for Map Name, Load Button, Save Button.
4. **Drawing Logic:**
   * If editor mode is active, clicking on the grid updates `GameMap` based on the selected brush.
   * Call `queue.write_buffer` to refresh the instanced rendering mesh when tiles are updated.
