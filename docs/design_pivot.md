# Design Pivot: 2D Isometric to 3D Isometric

## Context
The game was originally designed as a 2D isometric tower defense game using 2D sprites/tiles. However, to support more dynamic features, better visual quality, and smoother scaling/rotations, the project is pivoting to **3D objects** with an isometric camera perspective.

## Why 3D?
1. **Dynamic Lighting**: 3D meshes allow us to implement realistic lights and shadows, which is highly complex and performance-intensive with 2D sprites.
2. **Camera Freedom**: Although the camera remains locked in an isometric view by default, 3D allows for smooth zooming, rotation, and micro-adjustments.
3. **Asset Pipelines**: Creating and animating 3D assets in Blender is often more flexible than rendering out sheet sprites for every angle of every character/tower.
4. **Dynamic Geometry**: Drawing walls, pathways, and interactive environments dynamically is cleaner when working directly with 3D geometry rather than pre-rendered 2D tiles.

## Architectural Changes
- **Rendering Pipeline**: The render pipeline must support 3D vertices (coordinates containing X, Y, and Z) and shader modules configured for projection and camera matrices.
- **Renderer Context**: Refactoring the pipeline code out of the winit event handler state (`State`) into a dedicated `Renderer` context ensures modularity and prepares the engine for loading complex 3D meshes (OBJ/GLTF) in future phases.
- **Math Utilities**: Screen-to-world and world-to-screen functions will transition from simple 2D isometric conversions to 3D camera projection and raycasting.
