use crate::renderer::sprite::{Sprite, SpriteAlignment};
use crate::renderer::Vertex;

/// A single draw batch for rendering sprites that share a texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteDrawBatch {
    pub texture_id: usize,
    pub vertex_offset: u32,
    pub vertex_count: u32,
}

/// One of the six faces of an axis-aligned cube.
///
/// `North`/`South` are the faces perpendicular to the Z axis, `East`/`West`
/// perpendicular to X, and `Top`/`Bottom` perpendicular to Y. Used by terrain
/// meshing to emit only the faces that are actually visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Top,
    Bottom,
    North, // -Z
    South, // +Z
    East,  // +X
    West,  // -X
}

/// Represents an item in the sprite batch queue.
#[derive(Debug, Clone, PartialEq)]
pub enum BatchItem {
    Sprite {
        sprite: Sprite,
        alignment: SpriteAlignment,
    },
    CubeSides {
        position: glam::Vec3,
        size: glam::Vec3,
        texture_id: usize,
    },
    CubeTop {
        position: glam::Vec3,
        size: glam::Vec3,
        texture_id: usize,
    },
    /// A single cube face, used for face-culled terrain meshing so that
    /// hidden interior faces are never generated.
    CubeFace {
        position: glam::Vec3,
        size: glam::Vec3,
        face: Face,
        texture_id: usize,
    },
    /// A flat, upward-facing quad lying on the X-Z plane at `position.y`, drawn
    /// with an arbitrary (possibly translucent) `color`. Useful for ground
    /// decals (e.g. tower range indicators). Only `size.x` and `size.z` are used.
    OverlayQuad {
        position: glam::Vec3,
        size: glam::Vec3,
        color: [f32; 4],
        texture_id: usize,
    },
    /// A flat, upward-facing circle lying on the X-Z plane at `position.y`, drawn
    /// with an arbitrary (possibly translucent) `color`. Useful for ground decals
    /// (e.g. tower range indicators).
    OverlayCircle {
        position: glam::Vec3,
        radius: f32,
        color: [f32; 4],
        texture_id: usize,
    },
    /// A single cube face drawn with an arbitrary (possibly translucent)
    /// `color` instead of fixed directional shading. Used to build the 3D
    /// block-highlight shell.
    ColorFace {
        position: glam::Vec3,
        size: glam::Vec3,
        face: Face,
        color: [f32; 4],
        texture_id: usize,
    },
    /// Arbitrary 3D vertices from a transformed model.
    Model {
        vertices: Vec<Vertex>,
        texture_id: usize,
    },
}

impl BatchItem {
    pub fn texture_id(&self) -> usize {
        match self {
            BatchItem::Sprite { sprite, .. } => sprite.texture_id,
            BatchItem::CubeSides { texture_id, .. } => *texture_id,
            BatchItem::CubeTop { texture_id, .. } => *texture_id,
            BatchItem::CubeFace { texture_id, .. } => *texture_id,
            BatchItem::OverlayQuad { texture_id, .. } => *texture_id,
            BatchItem::OverlayCircle { texture_id, .. } => *texture_id,
            BatchItem::ColorFace { texture_id, .. } => *texture_id,
            BatchItem::Model { texture_id, .. } => *texture_id,
        }
    }

    pub fn generate_vertices(&self, vertices: &mut Vec<Vertex>) {
        match self {
            BatchItem::Sprite { sprite, alignment } => {
                vertices.extend_from_slice(&sprite.generate_vertices_aligned(*alignment));
            }
            BatchItem::CubeSides { position, size, .. } => {
                let x = position.x;
                let y = position.y;
                let z = position.z;
                let hx = size.x / 2.0;
                let hy = size.y / 2.0;
                let hz = size.z / 2.0;

                // 5 faces (all except top): front, back, left, right, bottom

                // Front face (Z = +hz)
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z + hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z + hz],
                    tex_coords: [0.0, 1.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z + hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z + hz],
                    tex_coords: [1.0, 0.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });

                // Back face (Z = -hz)
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z - hz],
                    tex_coords: [0.0, 1.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z - hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z - hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z - hz],
                    tex_coords: [1.0, 0.0],
                    color: [0.6, 0.6, 0.6, 1.0],
                });

                // Left face (X = -hx)
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z - hz],
                    tex_coords: [0.0, 1.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z + hz],
                    tex_coords: [1.0, 0.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });

                // Right face (X = +hx)
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z + hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z + hz],
                    tex_coords: [0.0, 1.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z - hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z + hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z - hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z - hz],
                    tex_coords: [1.0, 0.0],
                    color: [0.8, 0.8, 0.8, 1.0],
                });

                // Bottom face (Y = -hy)
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z + hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.4, 0.4, 0.4, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z - hz],
                    tex_coords: [0.0, 1.0],
                    color: [0.4, 0.4, 0.4, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z - hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.4, 0.4, 0.4, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y - hy, z + hz],
                    tex_coords: [0.0, 0.0],
                    color: [0.4, 0.4, 0.4, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z - hz],
                    tex_coords: [1.0, 1.0],
                    color: [0.4, 0.4, 0.4, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y - hy, z + hz],
                    tex_coords: [1.0, 0.0],
                    color: [0.4, 0.4, 0.4, 1.0],
                });
            }
            BatchItem::CubeTop { position, size, .. } => {
                let x = position.x;
                let y = position.y;
                let z = position.z;
                let hx = size.x / 2.0;
                let hy = size.y / 2.0;
                let hz = size.z / 2.0;

                // Top face only (Y = +hy)
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z + hz],
                    tex_coords: [0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [x - hx, y + hy, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [x + hx, y + hy, z - hz],
                    tex_coords: [1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
            BatchItem::CubeFace {
                position,
                size,
                face,
                ..
            } => {
                let x = position.x;
                let y = position.y;
                let z = position.z;
                let hx = size.x / 2.0;
                let hy = size.y / 2.0;
                let hz = size.z / 2.0;

                // Faces share the exact winding (CCW) and shading of the full
                // cube above, so a culled mesh is visually identical to the
                // old stacked-cube version. Side faces are tinted to fake
                // simple directional lighting.
                match face {
                    Face::Top => {
                        let c = [1.0, 1.0, 1.0, 1.0];
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z - hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z + hz],
                            tex_coords: [0.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z + hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z - hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z + hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z - hz],
                            tex_coords: [1.0, 0.0],
                            color: c,
                        });
                    }
                    Face::Bottom => {
                        let c = [0.4, 0.4, 0.4, 1.0];
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z + hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z - hz],
                            tex_coords: [0.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z - hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z + hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z - hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z + hz],
                            tex_coords: [1.0, 0.0],
                            color: c,
                        });
                    }
                    // South face (Z = +hz) — matches the cube "front" face.
                    Face::South => {
                        let c = [0.6, 0.6, 0.6, 1.0];
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z + hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z + hz],
                            tex_coords: [0.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z + hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z + hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z + hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z + hz],
                            tex_coords: [1.0, 0.0],
                            color: c,
                        });
                    }
                    // North face (Z = -hz) — matches the cube "back" face.
                    Face::North => {
                        let c = [0.6, 0.6, 0.6, 1.0];
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z - hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z - hz],
                            tex_coords: [0.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z - hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z - hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z - hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z - hz],
                            tex_coords: [1.0, 0.0],
                            color: c,
                        });
                    }
                    // West face (X = -hx) — matches the cube "left" face.
                    Face::West => {
                        let c = [0.8, 0.8, 0.8, 1.0];
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z - hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z - hz],
                            tex_coords: [0.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z + hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z - hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y - hy, z + hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x - hx, y + hy, z + hz],
                            tex_coords: [1.0, 0.0],
                            color: c,
                        });
                    }
                    // East face (X = +hx) — matches the cube "right" face.
                    Face::East => {
                        let c = [0.8, 0.8, 0.8, 1.0];
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z + hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z + hz],
                            tex_coords: [0.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z - hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z + hz],
                            tex_coords: [0.0, 0.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y - hy, z - hz],
                            tex_coords: [1.0, 1.0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x + hx, y + hy, z - hz],
                            tex_coords: [1.0, 0.0],
                            color: c,
                        });
                    }
                }
            }
            BatchItem::OverlayQuad {
                position,
                size,
                color,
                ..
            } => {
                let x = position.x;
                let y = position.y;
                let z = position.z;
                let hx = size.x / 2.0;
                let hz = size.z / 2.0;
                let c = *color;

                // Flat upward-facing quad at constant y, same CCW winding as a
                // cube top face so it isn't back-face culled.
                vertices.push(Vertex {
                    position: [x - hx, y, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: c,
                });
                vertices.push(Vertex {
                    position: [x - hx, y, z + hz],
                    tex_coords: [0.0, 1.0],
                    color: c,
                });
                vertices.push(Vertex {
                    position: [x + hx, y, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: c,
                });
                vertices.push(Vertex {
                    position: [x - hx, y, z - hz],
                    tex_coords: [0.0, 0.0],
                    color: c,
                });
                vertices.push(Vertex {
                    position: [x + hx, y, z + hz],
                    tex_coords: [1.0, 1.0],
                    color: c,
                });
                vertices.push(Vertex {
                    position: [x + hx, y, z - hz],
                    tex_coords: [1.0, 0.0],
                    color: c,
                });
            }
            BatchItem::OverlayCircle {
                position,
                radius,
                color,
                ..
            } => {
                let x = position.x;
                let y = position.y;
                let z = position.z;
                let r = *radius;
                let c = *color;

                let segments = 64;
                for i in 0..segments {
                    let theta1 = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
                    let theta2 = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (segments as f32);

                    let cos1 = theta1.cos();
                    let sin1 = theta1.sin();
                    let cos2 = theta2.cos();
                    let sin2 = theta2.sin();

                    let p1_x = x + r * cos1;
                    let p1_z = z + r * sin1;
                    let p2_x = x + r * cos2;
                    let p2_z = z + r * sin2;

                    // Filled circle (transparent white color `c`)
                    vertices.push(Vertex {
                        position: [x, y, z],
                        tex_coords: [0.5, 0.5],
                        color: c,
                    });
                    vertices.push(Vertex {
                        position: [p1_x, y, p1_z],
                        tex_coords: [0.5 + 0.5 * cos1, 0.5 + 0.5 * sin1],
                        color: c,
                    });
                    vertices.push(Vertex {
                        position: [p2_x, y, p2_z],
                        tex_coords: [0.5 + 0.5 * cos2, 0.5 + 0.5 * sin2],
                        color: c,
                    });
                }

                // Draw an outer ring/border for extra visual pop
                let ring_thickness = 0.006;
                let r_inner = r - ring_thickness;
                let border_color = [c[0], c[1], c[2], (c[3] * 8.0).min(0.8)];

                for i in 0..segments {
                    let theta1 = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
                    let theta2 = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (segments as f32);

                    let cos1 = theta1.cos();
                    let sin1 = theta1.sin();
                    let cos2 = theta2.cos();
                    let sin2 = theta2.sin();

                    let p1_outer_x = x + r * cos1;
                    let p1_outer_z = z + r * sin1;
                    let p2_outer_x = x + r * cos2;
                    let p2_outer_z = z + r * sin2;

                    let p1_inner_x = x + r_inner * cos1;
                    let p1_inner_z = z + r_inner * sin1;
                    let p2_inner_x = x + r_inner * cos2;
                    let p2_inner_z = z + r_inner * sin2;

                    // Triangle 1: p1_inner -> p1_outer -> p2_outer
                    vertices.push(Vertex {
                        position: [p1_inner_x, y, p1_inner_z],
                        tex_coords: [0.0, 0.0],
                        color: border_color,
                    });
                    vertices.push(Vertex {
                        position: [p1_outer_x, y, p1_outer_z],
                        tex_coords: [1.0, 0.0],
                        color: border_color,
                    });
                    vertices.push(Vertex {
                        position: [p2_outer_x, y, p2_outer_z],
                        tex_coords: [1.0, 1.0],
                        color: border_color,
                    });

                    // Triangle 2: p1_inner -> p2_outer -> p2_inner
                    vertices.push(Vertex {
                        position: [p1_inner_x, y, p1_inner_z],
                        tex_coords: [0.0, 0.0],
                        color: border_color,
                    });
                    vertices.push(Vertex {
                        position: [p2_outer_x, y, p2_outer_z],
                        tex_coords: [1.0, 1.0],
                        color: border_color,
                    });
                    vertices.push(Vertex {
                        position: [p2_inner_x, y, p2_inner_z],
                        tex_coords: [0.0, 1.0],
                        color: border_color,
                    });
                }
            }
            BatchItem::ColorFace {
                position,
                size,
                face,
                color,
                ..
            } => {
                push_color_face(vertices, *face, *position, *size, *color);
            }
            BatchItem::Model {
                vertices: model_verts,
                ..
            } => {
                vertices.extend_from_slice(model_verts);
            }
        }
    }
}

/// Pushes the six vertices of one cube `face` with a uniform `color`.
///
/// The geometry (positions, winding, UVs) is identical to [`BatchItem::CubeFace`];
/// only the color differs. Used to build translucent highlight shells.
fn push_color_face(
    vertices: &mut Vec<Vertex>,
    face: Face,
    position: glam::Vec3,
    size: glam::Vec3,
    color: [f32; 4],
) {
    let x = position.x;
    let y = position.y;
    let z = position.z;
    let hx = size.x / 2.0;
    let hy = size.y / 2.0;
    let hz = size.z / 2.0;
    let c = color;

    match face {
        Face::Top => {
            vertices.push(Vertex {
                position: [x - hx, y + hy, z - hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y + hy, z + hz],
                tex_coords: [0.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y + hy, z + hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y + hy, z - hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y + hy, z + hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y + hy, z - hz],
                tex_coords: [1.0, 0.0],
                color: c,
            });
        }
        Face::Bottom => {
            vertices.push(Vertex {
                position: [x - hx, y - hy, z + hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z - hz],
                tex_coords: [0.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z - hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z + hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z - hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z + hz],
                tex_coords: [1.0, 0.0],
                color: c,
            });
        }
        Face::South => {
            vertices.push(Vertex {
                position: [x - hx, y + hy, z + hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z + hz],
                tex_coords: [0.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z + hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y + hy, z + hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z + hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y + hy, z + hz],
                tex_coords: [1.0, 0.0],
                color: c,
            });
        }
        Face::North => {
            vertices.push(Vertex {
                position: [x + hx, y + hy, z - hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z - hz],
                tex_coords: [0.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z - hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y + hy, z - hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z - hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y + hy, z - hz],
                tex_coords: [1.0, 0.0],
                color: c,
            });
        }
        Face::West => {
            vertices.push(Vertex {
                position: [x - hx, y + hy, z - hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z - hz],
                tex_coords: [0.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z + hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y + hy, z - hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y - hy, z + hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x - hx, y + hy, z + hz],
                tex_coords: [1.0, 0.0],
                color: c,
            });
        }
        Face::East => {
            vertices.push(Vertex {
                position: [x + hx, y + hy, z + hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z + hz],
                tex_coords: [0.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z - hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y + hy, z + hz],
                tex_coords: [0.0, 0.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y - hy, z - hz],
                tex_coords: [1.0, 1.0],
                color: c,
            });
            vertices.push(Vertex {
                position: [x + hx, y + hy, z - hz],
                tex_coords: [1.0, 0.0],
                color: c,
            });
        }
    }
}

pub struct SpriteBatcher {
    items_by_texture: Vec<Vec<BatchItem>>,
    vertices: Vec<Vertex>,
    batches: Vec<SpriteDrawBatch>,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_buffer_capacity: usize,
}

impl SpriteBatcher {
    pub fn new() -> Self {
        let mut items_by_texture = Vec::with_capacity(16);
        for _ in 0..16 {
            items_by_texture.push(Vec::with_capacity(128));
        }
        Self {
            items_by_texture,
            vertices: Vec::new(),
            batches: Vec::new(),
            vertex_buffer: None,
            vertex_buffer_capacity: 0,
        }
    }

    /// Clears the batch queue for a new frame.
    pub fn clear(&mut self) {
        for items in &mut self.items_by_texture {
            items.clear();
        }
        self.vertices.clear();
        self.batches.clear();
    }

    fn add_item(&mut self, item: BatchItem) {
        let tex_id = item.texture_id();
        if tex_id >= self.items_by_texture.len() {
            self.items_by_texture.resize_with(tex_id + 1, || Vec::with_capacity(128));
        }
        self.items_by_texture[tex_id].push(item);
    }

    /// Adds a 2.5D/3D sprite to the batcher.
    pub fn add_sprite(&mut self, sprite: Sprite, alignment: SpriteAlignment) {
        self.add_item(BatchItem::Sprite { sprite, alignment });
    }

    /// Adds a 3D cube/block to the batcher with separate top and side textures.
    pub fn add_cube(
        &mut self,
        position: glam::Vec3,
        size: glam::Vec3,
        side_texture_id: usize,
        top_texture_id: usize,
    ) {
        self.add_item(BatchItem::CubeSides {
            position,
            size,
            texture_id: side_texture_id,
        });
        self.add_item(BatchItem::CubeTop {
            position,
            size,
            texture_id: top_texture_id,
        });
    }

    /// Adds a single cube face to the batcher. Used by face-culled terrain
    /// meshing to emit only the faces that are actually visible.
    pub fn add_face(
        &mut self,
        position: glam::Vec3,
        size: glam::Vec3,
        face: Face,
        texture_id: usize,
    ) {
        self.add_item(BatchItem::CubeFace {
            position,
            size,
            face,
            texture_id,
        });
    }

    /// Adds a flat, upward-facing overlay quad at `position.y` drawn with the
    /// given (possibly translucent) `color`. Useful for ground decals.
    pub fn add_overlay_quad(
        &mut self,
        position: glam::Vec3,
        size: glam::Vec3,
        color: [f32; 4],
        texture_id: usize,
    ) {
        self.add_item(BatchItem::OverlayQuad {
            position,
            size,
            color,
            texture_id,
        });
    }

    /// Adds a flat, upward-facing overlay circle at `position.y` drawn with the
    /// given (possibly translucent) `color`. Useful for range indicators.
    pub fn add_overlay_circle(
        &mut self,
        position: glam::Vec3,
        radius: f32,
        color: [f32; 4],
        texture_id: usize,
    ) {
        self.add_item(BatchItem::OverlayCircle {
            position,
            radius,
            color,
            texture_id,
        });
    }

    /// Adds a single cube face tinted with an arbitrary `color`.
    pub fn add_color_face(
        &mut self,
        position: glam::Vec3,
        size: glam::Vec3,
        face: Face,
        color: [f32; 4],
        texture_id: usize,
    ) {
        self.add_item(BatchItem::ColorFace {
            position,
            size,
            face,
            color,
            texture_id,
        });
    }

    /// Adds a translucent highlight "shell" around a box centred at `center`
    /// with the given `size` (the top face plus the four side faces; the
    /// bottom is omitted as it's never visible from the isometric camera).
    ///
    /// This works for anything occupying a box in the world — a terrain block,
    /// or a tower floating above the ground — so the same call highlights both.
    pub fn add_highlight_box(
        &mut self,
        center: glam::Vec3,
        size: glam::Vec3,
        color: [f32; 4],
        texture_id: usize,
    ) {
        for face in [Face::Top, Face::North, Face::South, Face::East, Face::West] {
            self.add_item(BatchItem::ColorFace {
                position: center,
                size,
                face,
                color,
                texture_id,
            });
        }
    }

    /// Adds pre-transformed model vertices to the batcher.
    pub fn add_model(&mut self, vertices: Vec<Vertex>, texture_id: usize) {
        self.add_item(BatchItem::Model {
            vertices,
            texture_id,
        });
    }

    /// Returns a slice of the compiled batches.
    pub fn batches(&self) -> &[SpriteDrawBatch] {
        &self.batches
    }

    /// Returns the GPU vertex buffer.
    pub fn vertex_buffer(&self) -> Option<&wgpu::Buffer> {
        self.vertex_buffer.as_ref()
    }

    /// Returns a slice of the compiled vertices.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Sorts queued items by texture ID and compiles them into batches on the CPU.
    pub fn compile_batches(&mut self) {
        self.vertices.clear();
        self.batches.clear();

        for (texture_id, items) in self.items_by_texture.iter().enumerate() {
            if items.is_empty() {
                continue;
            }

            let start_idx = self.vertices.len();
            for item in items {
                item.generate_vertices(&mut self.vertices);
            }
            let vertex_count = (self.vertices.len() - start_idx) as u32;

            if vertex_count > 0 {
                self.batches.push(SpriteDrawBatch {
                    texture_id,
                    vertex_offset: start_idx as u32,
                    vertex_count,
                });
            }
        }
    }

    /// Compiles batches and uploads the generated vertices to the dynamic GPU vertex buffer.
    pub fn finalize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.compile_batches();

        // Upload/write to GPU buffer
        let required_capacity = self.vertices.len();
        if required_capacity > self.vertex_buffer_capacity {
            // Allocate with some padding to minimize future re-allocations
            let new_capacity = required_capacity.next_power_of_two().max(512);
            let size_in_bytes =
                (new_capacity * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress;

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("SpriteBatcher Dynamic Vertex Buffer"),
                size: size_in_bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.vertex_buffer = Some(buffer);
            self.vertex_buffer_capacity = new_capacity;
        }

        if let Some(ref buffer) = self.vertex_buffer {
            if !self.vertices.is_empty() {
                queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.vertices));
            }
        }
    }
}

impl Default for SpriteBatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_item_texture_id() {
        let sprite = Sprite::new(glam::Vec3::ZERO, glam::Vec2::ONE, 42, 0.0);
        let item1 = BatchItem::Sprite {
            sprite,
            alignment: SpriteAlignment::Horizontal,
        };
        assert_eq!(item1.texture_id(), 42);

        let item2 = BatchItem::CubeSides {
            position: glam::Vec3::ZERO,
            size: glam::Vec3::ONE,
            texture_id: 100,
        };
        assert_eq!(item2.texture_id(), 100);

        let item3 = BatchItem::CubeTop {
            position: glam::Vec3::ZERO,
            size: glam::Vec3::ONE,
            texture_id: 55,
        };
        assert_eq!(item3.texture_id(), 55);
    }

    #[test]
    fn test_cube_vertex_generation() {
        let sides = BatchItem::CubeSides {
            position: glam::Vec3::new(1.0, 2.0, 3.0),
            size: glam::Vec3::new(2.0, 2.0, 2.0),
            texture_id: 0,
        };
        let top = BatchItem::CubeTop {
            position: glam::Vec3::new(1.0, 2.0, 3.0),
            size: glam::Vec3::new(2.0, 2.0, 2.0),
            texture_id: 1,
        };
        let mut vertices = Vec::new();
        sides.generate_vertices(&mut vertices);
        top.generate_vertices(&mut vertices);

        // Sides = 5 faces * 6 = 30, Top = 1 face * 6 = 6, Total = 36
        assert_eq!(vertices.len(), 36);

        // Check first vertex: front face TL = (x-hx, y+hy, z+hz) = (0.0, 3.0, 4.0)
        assert_eq!(vertices[0].position, [0.0, 3.0, 4.0]);
        assert_eq!(vertices[1].position, [0.0, 1.0, 4.0]);
        assert_eq!(vertices[2].position, [2.0, 1.0, 4.0]);
    }

    #[test]
    fn test_batcher_sorting_and_compilation() {
        let mut batcher = SpriteBatcher::new();

        // Add items: sprites with textures 2, 1, 2, and a cube with sides=0, top=1
        let s0 = Sprite::new(glam::Vec3::ZERO, glam::Vec2::ONE, 2, 0.0);
        let s1 = Sprite::new(glam::Vec3::ZERO, glam::Vec2::ONE, 1, 0.0);
        let s2 = Sprite::new(glam::Vec3::ZERO, glam::Vec2::ONE, 2, 0.0);

        batcher.add_sprite(s0, SpriteAlignment::Horizontal);
        batcher.add_sprite(s1, SpriteAlignment::Horizontal);
        batcher.add_sprite(s2, SpriteAlignment::Horizontal);
        batcher.add_cube(glam::Vec3::ZERO, glam::Vec3::ONE, 0, 1);

        batcher.compile_batches();

        let batches = batcher.batches();
        // After sorting by texture_id:
        // Batch 0: texture_id = 0 (CubeSides = 30 vertices)
        // Batch 1: texture_id = 1 (CubeTop=6 + sprite=6 = 12 vertices)
        // Batch 2: texture_id = 2 (2 sprites = 12 vertices)
        assert_eq!(batches.len(), 3);

        assert_eq!(batches[0].texture_id, 0);
        assert_eq!(batches[0].vertex_offset, 0);
        assert_eq!(batches[0].vertex_count, 30);

        assert_eq!(batches[1].texture_id, 1);
        assert_eq!(batches[1].vertex_offset, 30);
        assert_eq!(batches[1].vertex_count, 12);

        assert_eq!(batches[2].texture_id, 2);
        assert_eq!(batches[2].vertex_offset, 42);
        assert_eq!(batches[2].vertex_count, 12);

        // Total = 30 + 12 + 12 = 54
        assert_eq!(batcher.vertices.len(), 54);
    }

    #[test]
    fn test_cube_face_vertex_count() {
        // Every single face must emit exactly 6 vertices (two triangles).
        for face in [
            Face::Top,
            Face::Bottom,
            Face::North,
            Face::South,
            Face::East,
            Face::West,
        ] {
            let item = BatchItem::CubeFace {
                position: glam::Vec3::new(1.0, 2.0, 3.0),
                size: glam::Vec3::new(2.0, 2.0, 2.0),
                face,
                texture_id: 7,
            };
            assert_eq!(item.texture_id(), 7);

            let mut v = Vec::new();
            item.generate_vertices(&mut v);
            assert_eq!(v.len(), 6, "face {:?} should emit 6 vertices", face);
        }
    }

    #[test]
    fn test_cube_face_top_matches_cube_top() {
        // CubeFace::Top must be identical to the dedicated CubeTop face so a
        // culled mesh looks exactly like the old stacked-cube version.
        let pos = glam::Vec3::new(1.0, 2.0, 3.0);
        let size = glam::Vec3::new(2.0, 2.0, 2.0);

        let mut a = Vec::new();
        BatchItem::CubeTop {
            position: pos,
            size,
            texture_id: 0,
        }
        .generate_vertices(&mut a);
        let mut b = Vec::new();
        BatchItem::CubeFace {
            position: pos,
            size,
            face: Face::Top,
            texture_id: 0,
        }
        .generate_vertices(&mut b);

        let pa: Vec<_> = a.iter().map(|v| v.position).collect();
        let pb: Vec<_> = b.iter().map(|v| v.position).collect();
        assert_eq!(pa, pb);
    }

    #[test]
    fn test_cube_face_sides_match_cube_sides() {
        // The four side faces, combined, must reproduce the front/back/left/
        // right faces of CubeSides (which also includes a bottom face).
        let pos = glam::Vec3::new(1.0, 2.0, 3.0);
        let size = glam::Vec3::new(2.0, 2.0, 2.0);

        let mut full = Vec::new();
        BatchItem::CubeSides {
            position: pos,
            size,
            texture_id: 0,
        }
        .generate_vertices(&mut full);
        // CubeSides = front, back, left, right, bottom = 5 faces * 6 = 30.
        // The first 24 vertices are the four lateral faces (bottom is last).
        let lateral: Vec<_> = full.iter().take(24).map(|v| v.position).collect();

        let mut sides = Vec::new();
        // Order matters: CubeSides emits front(+Z), back(-Z), left(-X), right(+X).
        for face in [Face::South, Face::North, Face::West, Face::East] {
            BatchItem::CubeFace {
                position: pos,
                size,
                face,
                texture_id: 0,
            }
            .generate_vertices(&mut sides);
        }
        let sides: Vec<_> = sides.iter().map(|v| v.position).collect();

        assert_eq!(lateral, sides);
    }

    #[test]
    fn test_overlay_quad_vertices_and_color() {
        let color = [1.0, 0.9, 0.3, 0.45];
        let item = BatchItem::OverlayQuad {
            position: glam::Vec3::new(1.0, 0.5, 2.0),
            size: glam::Vec3::new(0.12, 0.0, 0.12),
            color,
            texture_id: 5,
        };
        assert_eq!(item.texture_id(), 5);

        let mut v = Vec::new();
        item.generate_vertices(&mut v);
        assert_eq!(v.len(), 6, "overlay quad is two triangles");
        // The translucent color must be carried through to every vertex, and
        // the quad must be flat (constant y).
        for vert in &v {
            assert_eq!(vert.color, color);
            assert_eq!(vert.position[1], 0.5);
        }
    }

    #[test]
    fn test_color_face_matches_cube_face_geometry() {
        // ColorFace must have identical geometry to CubeFace; only color differs.
        let pos = glam::Vec3::new(1.0, 2.0, 3.0);
        let size = glam::Vec3::new(2.0, 2.0, 2.0);
        let tint = [1.0, 0.9, 0.3, 0.45];

        for face in [
            Face::Top,
            Face::North,
            Face::South,
            Face::East,
            Face::West,
            Face::Bottom,
        ] {
            let mut a = Vec::new();
            BatchItem::CubeFace {
                position: pos,
                size,
                face,
                texture_id: 0,
            }
            .generate_vertices(&mut a);
            let mut b = Vec::new();
            BatchItem::ColorFace {
                position: pos,
                size,
                face,
                color: tint,
                texture_id: 0,
            }
            .generate_vertices(&mut b);

            let pa: Vec<_> = a.iter().map(|v| v.position).collect();
            let pb: Vec<_> = b.iter().map(|v| v.position).collect();
            assert_eq!(pa, pb, "geometry mismatch for {:?}", face);

            for v in &b {
                assert_eq!(v.color, tint, "color not applied for {:?}", face);
            }
        }
    }

    #[test]
    fn test_highlight_box_emits_five_faces() {
        let mut batcher = SpriteBatcher::new();
        batcher.add_highlight_box(
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::splat(0.12),
            [1.0, 0.9, 0.3, 0.45],
            5,
        );
        batcher.compile_batches();

        let batches = batcher.batches();
        assert_eq!(batches.len(), 1, "single texture -> one batch");
        // 5 faces (top + 4 sides), 6 vertices each = 30. No bottom face.
        assert_eq!(batches[0].vertex_count, 30);
        assert_eq!(batches[0].texture_id, 5);
    }

    #[test]
    fn test_add_face_queues_and_batches() {
        let mut batcher = SpriteBatcher::new();
        batcher.add_face(glam::Vec3::ZERO, glam::Vec3::ONE, Face::East, 4);
        batcher.add_face(glam::Vec3::ZERO, glam::Vec3::ONE, Face::Top, 4);
        batcher.compile_batches();

        let batches = batcher.batches();
        assert_eq!(
            batches.len(),
            1,
            "same texture_id should merge into one batch"
        );
        assert_eq!(batches[0].texture_id, 4);
        assert_eq!(batches[0].vertex_count, 12);
    }

    #[test]
    fn test_add_overlay_circle() {
        let mut batcher = SpriteBatcher::new();
        let color = [1.0, 1.0, 1.0, 0.05];
        batcher.add_overlay_circle(glam::Vec3::new(1.0, 2.0, 3.0), 5.0, color, 7);
        batcher.compile_batches();

        let batches = batcher.batches();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].texture_id, 7);
        // Filled circle: 64 segments * 3 vertices = 192 vertices.
        // Outer ring: 64 segments * 6 vertices = 384 vertices.
        // Total: 192 + 384 = 576 vertices.
        assert_eq!(batches[0].vertex_count, 576);
    }
}
