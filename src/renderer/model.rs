use glam::{Mat4, Quat, Vec3};
use std::path::Path;
use tobj;

use crate::renderer::Vertex;

/// Represents a loaded 3D model with its geometry.
#[derive(Debug, Clone)]
pub struct Model {
    /// The base vertices parsed from the model
    pub vertices: Vec<Vertex>,
}

impl Model {
    /// Loads a `.obj` model from the given path.
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // Load the OBJ file using tobj
        let (models, _materials) = tobj::load_obj(
            path.as_ref(),
            &tobj::LoadOptions {
                single_index: true,     // Convert everything to a single index buffer layout
                triangulate: true,      // Triangulate all polygons
                ignore_points: true,
                ignore_lines: true,
            },
        )?;

        let mut vertices = Vec::new();

        for model in models {
            let mesh = &model.mesh;
            
            // Loop through each triangle
            for i in 0..mesh.indices.len() / 3 {
                for j in 0..3 {
                    let idx = mesh.indices[i * 3 + j] as usize;
                    
                    let pos = [
                        mesh.positions[idx * 3],
                        mesh.positions[idx * 3 + 1],
                        mesh.positions[idx * 3 + 2],
                    ];
                    
                    // tobj guarantees texcoords are populated if the file had them,
                    // but we should provide a fallback just in case.
                    let tex_coords = if !mesh.texcoords.is_empty() {
                        [
                            mesh.texcoords[idx * 2],
                            // V-coordinate in OBJ goes from bottom to top, but wgpu expects top to bottom.
                            // However, we'll leave it as is, or flip it if the texture is inverted.
                            1.0 - mesh.texcoords[idx * 2 + 1],
                        ]
                    } else {
                        [0.0, 0.0]
                    };
                    
                    vertices.push(Vertex {
                        position: pos,
                        tex_coords,
                        color: [1.0, 1.0, 1.0, 1.0], // Default color
                    });
                }
            }
        }

        Ok(Self { vertices })
    }

    /// Transforms the model's base vertices and returns them ready for batching.
    /// Applies translation, uniform scale, and rotation around the Y-axis.
    pub fn generate_vertices(
        &self,
        position: Vec3,
        scale: f32,
        rotation_y: f32,
        color: [f32; 4],
    ) -> Vec<Vertex> {
        let rotation = Quat::from_rotation_y(rotation_y);
        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            rotation,
            position,
        );

        self.vertices
            .iter()
            .map(|v| {
                let local_pos = Vec3::from(v.position);
                let world_pos = transform.transform_point3(local_pos);
                
                Vertex {
                    position: world_pos.into(),
                    tex_coords: v.tex_coords,
                    color,
                }
            })
            .collect()
    }
}
