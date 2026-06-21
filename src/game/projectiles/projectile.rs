use glam::Vec2;

pub struct Projectile {
    pub start_position: Vec2,
    pub position: Vec2,
    pub target_position: Vec2,
    pub speed: f32,
    pub damage: f32,
    pub spawn_height_offset: f32,
    pub alive: bool,
}

impl Projectile {
    pub fn new(position: Vec2, target_position: Vec2, speed: f32, damage: f32, spawn_height_offset: f32) -> Self {
        Self {
            start_position: position,
            position,
            target_position,
            speed,
            damage,
            spawn_height_offset,
            alive: true,
        }
    }

    /// Update projectile position. Returns true if the projectile has reached the target.
    pub fn update(&mut self, dt: f32) -> bool {
        if !self.alive {
            return true;
        }

        let direction = self.target_position - self.position;
        let distance = direction.length();

        let travel_distance = self.speed * dt;

        if travel_distance >= distance {
            // Reached target
            self.position = self.target_position;
            self.alive = false;
            return true;
        }

        self.position += direction.normalize() * travel_distance;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projectile_movement() {
        let mut proj = Projectile::new(Vec2::ZERO, Vec2::new(10.0, 0.0), 5.0, 10.0, 0.8);
        
        let reached = proj.update(1.0);
        assert!(!reached);
        assert_eq!(proj.position.x, 5.0);
        assert!(proj.alive);

        let reached = proj.update(1.0);
        assert!(reached);
        assert_eq!(proj.position.x, 10.0);
        assert!(!proj.alive);
    }
}
