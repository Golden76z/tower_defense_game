use glam::{Vec2, Vec3};

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
}
