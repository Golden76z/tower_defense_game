use crate::renderer::Vertex;
use crate::renderer::sprite::{Sprite, SpriteAlignment};

/// A single draw batch for rendering sprites that share a texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteDrawBatch {
    pub texture_id: usize,
    pub vertex_offset: u32,
    pub vertex_count: u32,
}

/// Represents an item in the sprite batch queue.
#[derive(Debug, Clone, PartialEq)]
pub enum BatchItem {
    Sprite {
        sprite: Sprite,
        alignment: SpriteAlignment,
    },
    Cube {
        position: glam::Vec3,
        size: glam::Vec3,
        texture_id: usize,
    },
}

impl BatchItem {
    pub fn texture_id(&self) -> usize {
        match self {
            BatchItem::Sprite { sprite, .. } => sprite.texture_id,
            BatchItem::Cube { texture_id, .. } => *texture_id,
        }
    }

    pub fn generate_vertices(&self, vertices: &mut Vec<Vertex>) {
        match self {
            BatchItem::Sprite { sprite, alignment } => {
                vertices.extend_from_slice(&sprite.generate_vertices_aligned(*alignment));
            }
            BatchItem::Cube { position, size, .. } => {
                let x = position.x;
                let y = position.y;
                let z = position.z;
                let hx = size.x / 2.0;
                let hy = size.y / 2.0;
                let hz = size.z / 2.0;

                // Push 36 vertices (6 faces * 6 vertices) forming triangles with CCW winding when viewed from outside.
                
                // Front face (Z = +hz) - Bottom Left face in isometric projection
                vertices.push(Vertex { position: [x - hx, y + hy, z + hz], tex_coords: [0.0, 0.0], color: [0.6, 0.6, 0.6, 1.0] }); // TL
                vertices.push(Vertex { position: [x - hx, y - hy, z + hz], tex_coords: [0.0, 1.0], color: [0.6, 0.6, 0.6, 1.0] }); // BL
                vertices.push(Vertex { position: [x + hx, y - hy, z + hz], tex_coords: [1.0, 1.0], color: [0.6, 0.6, 0.6, 1.0] }); // BR
                vertices.push(Vertex { position: [x - hx, y + hy, z + hz], tex_coords: [0.0, 0.0], color: [0.6, 0.6, 0.6, 1.0] }); // TL
                vertices.push(Vertex { position: [x + hx, y - hy, z + hz], tex_coords: [1.0, 1.0], color: [0.6, 0.6, 0.6, 1.0] }); // BR
                vertices.push(Vertex { position: [x + hx, y + hy, z + hz], tex_coords: [1.0, 0.0], color: [0.6, 0.6, 0.6, 1.0] }); // TR

                // Back face (Z = -hz)
                vertices.push(Vertex { position: [x + hx, y + hy, z - hz], tex_coords: [0.0, 0.0], color: [0.6, 0.6, 0.6, 1.0] }); // TL
                vertices.push(Vertex { position: [x + hx, y - hy, z - hz], tex_coords: [0.0, 1.0], color: [0.6, 0.6, 0.6, 1.0] }); // BL
                vertices.push(Vertex { position: [x - hx, y - hy, z - hz], tex_coords: [1.0, 1.0], color: [0.6, 0.6, 0.6, 1.0] }); // BR
                vertices.push(Vertex { position: [x + hx, y + hy, z - hz], tex_coords: [0.0, 0.0], color: [0.6, 0.6, 0.6, 1.0] }); // TL
                vertices.push(Vertex { position: [x - hx, y - hy, z - hz], tex_coords: [1.0, 1.0], color: [0.6, 0.6, 0.6, 1.0] }); // BR
                vertices.push(Vertex { position: [x - hx, y + hy, z - hz], tex_coords: [1.0, 0.0], color: [0.6, 0.6, 0.6, 1.0] }); // TR

                // Left face (X = -hx)
                vertices.push(Vertex { position: [x - hx, y + hy, z - hz], tex_coords: [0.0, 0.0], color: [0.8, 0.8, 0.8, 1.0] }); // TL
                vertices.push(Vertex { position: [x - hx, y - hy, z - hz], tex_coords: [0.0, 1.0], color: [0.8, 0.8, 0.8, 1.0] }); // BL
                vertices.push(Vertex { position: [x - hx, y - hy, z + hz], tex_coords: [1.0, 1.0], color: [0.8, 0.8, 0.8, 1.0] }); // BR
                vertices.push(Vertex { position: [x - hx, y + hy, z - hz], tex_coords: [0.0, 0.0], color: [0.8, 0.8, 0.8, 1.0] }); // TL
                vertices.push(Vertex { position: [x - hx, y - hy, z + hz], tex_coords: [1.0, 1.0], color: [0.8, 0.8, 0.8, 1.0] }); // BR
                vertices.push(Vertex { position: [x - hx, y + hy, z + hz], tex_coords: [1.0, 0.0], color: [0.8, 0.8, 0.8, 1.0] }); // TR

                // Right face (X = +hx) - Bottom Right face in isometric projection
                vertices.push(Vertex { position: [x + hx, y + hy, z + hz], tex_coords: [0.0, 0.0], color: [0.8, 0.8, 0.8, 1.0] }); // TL
                vertices.push(Vertex { position: [x + hx, y - hy, z + hz], tex_coords: [0.0, 1.0], color: [0.8, 0.8, 0.8, 1.0] }); // BL
                vertices.push(Vertex { position: [x + hx, y - hy, z - hz], tex_coords: [1.0, 1.0], color: [0.8, 0.8, 0.8, 1.0] }); // BR
                vertices.push(Vertex { position: [x + hx, y + hy, z + hz], tex_coords: [0.0, 0.0], color: [0.8, 0.8, 0.8, 1.0] }); // TL
                vertices.push(Vertex { position: [x + hx, y - hy, z - hz], tex_coords: [1.0, 1.0], color: [0.8, 0.8, 0.8, 1.0] }); // BR
                vertices.push(Vertex { position: [x + hx, y + hy, z - hz], tex_coords: [1.0, 0.0], color: [0.8, 0.8, 0.8, 1.0] }); // TR

                // Top face (Y = +hy) - Up face in isometric projection
                vertices.push(Vertex { position: [x - hx, y + hy, z - hz], tex_coords: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] }); // TL
                vertices.push(Vertex { position: [x - hx, y + hy, z + hz], tex_coords: [0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] }); // BL
                vertices.push(Vertex { position: [x + hx, y + hy, z + hz], tex_coords: [1.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] }); // BR
                vertices.push(Vertex { position: [x - hx, y + hy, z - hz], tex_coords: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] }); // TL
                vertices.push(Vertex { position: [x + hx, y + hy, z + hz], tex_coords: [1.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] }); // BR
                vertices.push(Vertex { position: [x + hx, y + hy, z - hz], tex_coords: [1.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] }); // TR

                // Bottom face (Y = -hy)
                vertices.push(Vertex { position: [x - hx, y - hy, z + hz], tex_coords: [0.0, 0.0], color: [0.4, 0.4, 0.4, 1.0] }); // TL
                vertices.push(Vertex { position: [x - hx, y - hy, z - hz], tex_coords: [0.0, 1.0], color: [0.4, 0.4, 0.4, 1.0] }); // BL
                vertices.push(Vertex { position: [x + hx, y - hy, z - hz], tex_coords: [1.0, 1.0], color: [0.4, 0.4, 0.4, 1.0] }); // BR
                vertices.push(Vertex { position: [x - hx, y - hy, z + hz], tex_coords: [0.0, 0.0], color: [0.4, 0.4, 0.4, 1.0] }); // TL
                vertices.push(Vertex { position: [x + hx, y - hy, z - hz], tex_coords: [1.0, 1.0], color: [0.4, 0.4, 0.4, 1.0] }); // BR
                vertices.push(Vertex { position: [x + hx, y - hy, z + hz], tex_coords: [1.0, 0.0], color: [0.4, 0.4, 0.4, 1.0] }); // TR
            }
        }
    }
}

pub struct SpriteBatcher {
    items: Vec<BatchItem>,
    vertices: Vec<Vertex>,
    batches: Vec<SpriteDrawBatch>,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_buffer_capacity: usize,
}

impl SpriteBatcher {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            vertices: Vec::new(),
            batches: Vec::new(),
            vertex_buffer: None,
            vertex_buffer_capacity: 0,
        }
    }

    /// Clears the batch queue for a new frame.
    pub fn clear(&mut self) {
        self.items.clear();
        self.vertices.clear();
        self.batches.clear();
    }

    /// Adds a 2.5D/3D sprite to the batcher.
    pub fn add_sprite(&mut self, sprite: Sprite, alignment: SpriteAlignment) {
        self.items.push(BatchItem::Sprite { sprite, alignment });
    }

    /// Adds a 3D cube/block to the batcher.
    pub fn add_cube(&mut self, position: glam::Vec3, size: glam::Vec3, texture_id: usize) {
        self.items.push(BatchItem::Cube { position, size, texture_id });
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
        // Sort items by texture_id to minimize state changes
        self.items.sort_by_key(|item| item.texture_id());

        self.vertices.clear();
        self.batches.clear();

        if self.items.is_empty() {
            return;
        }

        let mut current_batch: Option<SpriteDrawBatch> = None;

        for item in &self.items {
            let texture_id = item.texture_id();
            let start_idx = self.vertices.len();
            
            // Generate vertices for the item
            item.generate_vertices(&mut self.vertices);
            
            let vertex_count = (self.vertices.len() - start_idx) as u32;

            if let Some(ref mut batch) = current_batch {
                if batch.texture_id == texture_id {
                    batch.vertex_count += vertex_count;
                } else {
                    self.batches.push(batch.clone());
                    current_batch = Some(SpriteDrawBatch {
                        texture_id,
                        vertex_offset: start_idx as u32,
                        vertex_count,
                    });
                }
            } else {
                current_batch = Some(SpriteDrawBatch {
                    texture_id,
                    vertex_offset: start_idx as u32,
                    vertex_count,
                });
            }
        }

        if let Some(batch) = current_batch {
            self.batches.push(batch);
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
            let size_in_bytes = (new_capacity * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress;

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
        let item1 = BatchItem::Sprite { sprite, alignment: SpriteAlignment::Horizontal };
        assert_eq!(item1.texture_id(), 42);

        let item2 = BatchItem::Cube {
            position: glam::Vec3::ZERO,
            size: glam::Vec3::ONE,
            texture_id: 100,
        };
        assert_eq!(item2.texture_id(), 100);
    }

    #[test]
    fn test_cube_vertex_generation() {
        let item = BatchItem::Cube {
            position: glam::Vec3::new(1.0, 2.0, 3.0),
            size: glam::Vec3::new(2.0, 2.0, 2.0),
            texture_id: 0,
        };
        let mut vertices = Vec::new();
        item.generate_vertices(&mut vertices);
        
        // Cube must have 36 vertices (6 faces * 6 vertices)
        assert_eq!(vertices.len(), 36);

        // Check a couple of points to verify translation/size
        // Corner 0 is (x - hx, y + hy, z + hz) = (1 - 1, 2 + 1, 3 + 1) = (0.0, 3.0, 4.0)
        assert_eq!(vertices[0].position, [0.0, 3.0, 4.0]);
        // Corner 1 is (x - hx, y - hy, z + hz) = (0.0, 1.0, 4.0)
        assert_eq!(vertices[1].position, [0.0, 1.0, 4.0]);
        // Corner 2 is (x + hx, y - hy, z + hz) = (2.0, 1.0, 4.0)
        assert_eq!(vertices[2].position, [2.0, 1.0, 4.0]);
    }

    #[test]
    fn test_batcher_sorting_and_compilation() {
        let mut batcher = SpriteBatcher::new();

        // Add items out of order of texture_id: 2, 1, 2, 0
        let s0 = Sprite::new(glam::Vec3::ZERO, glam::Vec2::ONE, 2, 0.0);
        let s1 = Sprite::new(glam::Vec3::ZERO, glam::Vec2::ONE, 1, 0.0);
        let s2 = Sprite::new(glam::Vec3::ZERO, glam::Vec2::ONE, 2, 0.0);
        let c3 = glam::Vec3::ZERO; // Cube with texture 0

        batcher.add_sprite(s0, SpriteAlignment::Horizontal);
        batcher.add_sprite(s1, SpriteAlignment::Horizontal);
        batcher.add_sprite(s2, SpriteAlignment::Horizontal);
        batcher.add_cube(c3, glam::Vec3::ONE, 0);

        // Compile batches on CPU
        batcher.compile_batches();

        let batches = batcher.batches();
        // Since we sort by texture_id, order of batches should be:
        // Batch 0: texture_id = 0 (1 cube = 36 vertices)
        // Batch 1: texture_id = 1 (1 sprite = 6 vertices)
        // Batch 2: texture_id = 2 (2 sprites = 12 vertices)
        assert_eq!(batches.len(), 3);

        assert_eq!(batches[0].texture_id, 0);
        assert_eq!(batches[0].vertex_offset, 0);
        assert_eq!(batches[0].vertex_count, 36);

        assert_eq!(batches[1].texture_id, 1);
        assert_eq!(batches[1].vertex_offset, 36);
        assert_eq!(batches[1].vertex_count, 6);

        assert_eq!(batches[2].texture_id, 2);
        assert_eq!(batches[2].vertex_offset, 42);
        assert_eq!(batches[2].vertex_count, 12);
        
        // Total vertices generated = 36 + 6 + 12 = 54
        assert_eq!(batcher.vertices.len(), 54);
    }
}
