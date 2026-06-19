use glam::{Vec2, Vec3};
use crate::renderer::Vertex;

/// Specifies how the sprite is aligned in the 3D isometric world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteAlignment {
    /// Lying flat on the X-Z ground plane.
    Horizontal,
    /// Standing upright on the X-Y plane.
    Vertical,
}

/// Represents a sprite in the 3D isometric world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub position: Vec3,
    pub size: Vec2,
    pub texture_id: usize,
    pub rotation: f32,
}

impl Sprite {
    /// Creates a new `Sprite` with the given parameters.
    pub fn new(position: Vec3, size: Vec2, texture_id: usize, rotation: f32) -> Self {
        Self {
            position,
            size,
            texture_id,
            rotation,
        }
    }

    /// Returns the sprite's position.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Sets the sprite's position.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Translates the sprite's position by the given offset.
    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
    }

    /// Returns the sprite's size.
    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// Sets the sprite's size.
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    /// Returns the sprite's texture ID.
    pub fn texture_id(&self) -> usize {
        self.texture_id
    }

    /// Sets the sprite's texture ID.
    pub fn set_texture_id(&mut self, texture_id: usize) {
        self.texture_id = texture_id;
    }

    /// Returns the sprite's rotation in radians.
    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    /// Sets the sprite's rotation in radians.
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    /// Rotates the sprite by the given angle (in radians).
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }

    /// Generates 6 vertices (2 triangles) representing the sprite quad,
    /// defaulting to `SpriteAlignment::Horizontal` (lying flat on the X-Z ground plane).
    pub fn generate_vertices(&self) -> [Vertex; 6] {
        self.generate_vertices_aligned(SpriteAlignment::Horizontal)
    }

    /// Generates 6 vertices (2 triangles) representing the sprite quad with a specific alignment.
    pub fn generate_vertices_aligned(&self, alignment: SpriteAlignment) -> [Vertex; 6] {
        let half_w = self.size.x / 2.0;
        let half_h = self.size.y / 2.0;

        // 1. Calculate the 4 corners in 2D local space
        let tl_2d = rotate_2d_point(-half_w, half_h, self.rotation);
        let bl_2d = rotate_2d_point(-half_w, -half_h, self.rotation);
        let br_2d = rotate_2d_point(half_w, -half_h, self.rotation);
        let tr_2d = rotate_2d_point(half_w, half_h, self.rotation);

        // 2. Project corners into 3D space based on alignment and translate by position
        let (tl_pos, bl_pos, br_pos, tr_pos) = match alignment {
            SpriteAlignment::Horizontal => {
                let tl = [self.position.x + tl_2d.x, self.position.y, self.position.z + tl_2d.y];
                let bl = [self.position.x + bl_2d.x, self.position.y, self.position.z + bl_2d.y];
                let br = [self.position.x + br_2d.x, self.position.y, self.position.z + br_2d.y];
                let tr = [self.position.x + tr_2d.x, self.position.y, self.position.z + tr_2d.y];
                (tl, bl, br, tr)
            }
            SpriteAlignment::Vertical => {
                let tl = [self.position.x + tl_2d.x, self.position.y + tl_2d.y, self.position.z];
                let bl = [self.position.x + bl_2d.x, self.position.y + bl_2d.y, self.position.z];
                let br = [self.position.x + br_2d.x, self.position.y + br_2d.y, self.position.z];
                let tr = [self.position.x + tr_2d.x, self.position.y + tr_2d.y, self.position.z];
                (tl, bl, br, tr)
            }
        };

        // 3. Assemble into two triangles with matching winding order (Ccw after projection) and correct UVs.
        // Triangle 1: Top-Left -> Bottom-Right -> Bottom-Left
        // Triangle 2: Top-Left -> Top-Right -> Bottom-Right
        [
            // Triangle 1
            Vertex {
                position: tl_pos,
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: br_pos,
                tex_coords: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: bl_pos,
                tex_coords: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            // Triangle 2
            Vertex {
                position: tl_pos,
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: tr_pos,
                tex_coords: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: br_pos,
                tex_coords: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ]
    }
}

/// Rotates a 2D point by the given angle (in radians) around the origin.
fn rotate_2d_point(x: f32, y: f32, angle_rad: f32) -> Vec2 {
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    Vec2::new(
        x * cos - y * sin,
        x * sin + y * cos,
    )
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            size: Vec2::ONE,
            texture_id: 0,
            rotation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_sprite_default_creation() {
        let sprite = Sprite::default();
        assert_eq!(sprite.position(), Vec3::ZERO);
        assert_eq!(sprite.size(), Vec2::ONE);
        assert_eq!(sprite.texture_id(), 0);
        assert_eq!(sprite.rotation(), 0.0);
    }

    #[test]
    fn test_sprite_custom_creation() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let size = Vec2::new(4.0, 5.0);
        let sprite = Sprite::new(pos, size, 42, 1.5);
        assert_eq!(sprite.position(), pos);
        assert_eq!(sprite.size(), size);
        assert_eq!(sprite.texture_id(), 42);
        assert_eq!(sprite.rotation(), 1.5);
    }

    #[test]
    fn test_sprite_position_getters_setters() {
        let mut sprite = Sprite::default();
        let new_pos = Vec3::new(-1.0, 5.5, 10.0);
        sprite.set_position(new_pos);
        assert_eq!(sprite.position(), new_pos);
        
        let translation = Vec3::new(2.0, -1.0, 0.5);
        sprite.translate(translation);
        assert_eq!(sprite.position(), Vec3::new(1.0, 4.5, 10.5));
    }

    #[test]
    fn test_sprite_transformations() {
        let mut sprite = Sprite::default();
        
        sprite.set_size(Vec2::new(2.0, 3.0));
        assert_eq!(sprite.size(), Vec2::new(2.0, 3.0));
        
        sprite.set_texture_id(5);
        assert_eq!(sprite.texture_id(), 5);
        
        sprite.set_rotation(1.0);
        assert_eq!(sprite.rotation(), 1.0);
        
        sprite.rotate(0.5);
        assert_eq!(sprite.rotation(), 1.5);
    }

    #[test]
    fn test_vertex_generation_default() {
        let sprite = Sprite::default(); // pos: (0,0,0), size: (1,1), rot: 0
        let vertices = sprite.generate_vertices();

        // Check UV coords
        assert_eq!(vertices[0].tex_coords, [0.0, 0.0]); // TL
        assert_eq!(vertices[1].tex_coords, [1.0, 1.0]); // BR
        assert_eq!(vertices[2].tex_coords, [0.0, 1.0]); // BL
        assert_eq!(vertices[3].tex_coords, [0.0, 0.0]); // TL
        assert_eq!(vertices[4].tex_coords, [1.0, 0.0]); // TR
        assert_eq!(vertices[5].tex_coords, [1.0, 1.0]); // BR

        // Check Positions (Horizontal/ground plane X-Z, centered around (0,0,0))
        // TL should be [-0.5, 0.0, 0.5]
        assert_relative_eq!(vertices[0].position[0], -0.5);
        assert_relative_eq!(vertices[0].position[1], 0.0);
        assert_relative_eq!(vertices[0].position[2], 0.5);

        // BR should be [0.5, 0.0, -0.5]
        assert_relative_eq!(vertices[1].position[0], 0.5);
        assert_relative_eq!(vertices[1].position[1], 0.0);
        assert_relative_eq!(vertices[1].position[2], -0.5);

        // BL should be [-0.5, 0.0, -0.5]
        assert_relative_eq!(vertices[2].position[0], -0.5);
        assert_relative_eq!(vertices[2].position[1], 0.0);
        assert_relative_eq!(vertices[2].position[2], -0.5);

        // TR should be [0.5, 0.0, 0.5]
        assert_relative_eq!(vertices[4].position[0], 0.5);
        assert_relative_eq!(vertices[4].position[1], 0.0);
        assert_relative_eq!(vertices[4].position[2], 0.5);
    }

    #[test]
    fn test_vertex_generation_vertical() {
        let sprite = Sprite::new(Vec3::new(1.0, 2.0, 3.0), Vec2::new(2.0, 4.0), 0, 0.0);
        let vertices = sprite.generate_vertices_aligned(SpriteAlignment::Vertical);

        // Size 2x4, position (1, 2, 3), vertical (X-Y plane)
        // TL local: (-1, 2, 0) -> world: (0, 4, 3)
        assert_relative_eq!(vertices[0].position[0], 0.0);
        assert_relative_eq!(vertices[0].position[1], 4.0);
        assert_relative_eq!(vertices[0].position[2], 3.0);

        // BR local: (1, -2, 0) -> world: (2, 0, 3)
        assert_relative_eq!(vertices[1].position[0], 2.0);
        assert_relative_eq!(vertices[1].position[1], 0.0);
        assert_relative_eq!(vertices[1].position[2], 3.0);

        // BL local: (-1, -2, 0) -> world: (0, 0, 3)
        assert_relative_eq!(vertices[2].position[0], 0.0);
        assert_relative_eq!(vertices[2].position[1], 0.0);
        assert_relative_eq!(vertices[2].position[2], 3.0);

        // TR local: (1, 2, 0) -> world: (2, 4, 3)
        assert_relative_eq!(vertices[4].position[0], 2.0);
        assert_relative_eq!(vertices[4].position[1], 4.0);
        assert_relative_eq!(vertices[4].position[2], 3.0);
    }

    #[test]
    fn test_vertex_generation_rotation() {
        use std::f32::consts::FRAC_PI_2;
        
        let sprite = Sprite::new(Vec3::ZERO, Vec2::new(2.0, 2.0), 0, FRAC_PI_2);
        let vertices = sprite.generate_vertices(); // Horizontal (X-Z plane)

        // Size 2x2. rotated 90 degrees CCW in local space.
        // Unrotated local TL was (-1, 1).
        // Rotated local TL: (-1*0 - 1*1, -1*1 + 1*0) = (-1, -1)
        // Let's verify mathematically:
        // rotate_2d_point(-1, 1, PI/2) = ( -1*cos(PI/2) - 1*sin(PI/2), -1*sin(PI/2) + 1*cos(PI/2) )
        // = ( -1*0 - 1*1, -1*1 + 1*0 ) = (-1, -1)
        assert_relative_eq!(vertices[0].position[0], -1.0, epsilon = 1e-6);
        assert_relative_eq!(vertices[0].position[1], 0.0);
        assert_relative_eq!(vertices[0].position[2], -1.0, epsilon = 1e-6);
    }
}
