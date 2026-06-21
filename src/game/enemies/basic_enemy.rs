use glam::Vec2;
use crate::game::map::path::Path;
use crate::game::enemies::enemy_base::Enemy;

/// A basic enemy type with simple path following and health.
pub struct BasicEnemy {
    position: Vec2,
    health: f32,
    speed: f32,
    reward: u32,
    waypoint_index: usize,
}

impl BasicEnemy {
    /// Creates a new basic enemy at the given starting position.
    pub fn new(start_position: Vec2) -> Self {
        Self {
            position: start_position,
            health: 100.0,
            speed: 2.0,
            reward: 10,
            waypoint_index: 1, // Start by moving to the second waypoint
        }
    }

    /// Gets the gold reward for defeating this enemy.
    pub fn reward(&self) -> u32 {
        self.reward
    }
}

impl Enemy for BasicEnemy {
    fn update(&mut self, dt: f32, path: &Path) {
        if path.is_finished(self.waypoint_index) {
            return;
        }

        // Check if we reached the current waypoint
        if path.reached_waypoint(self.position, self.waypoint_index) {
            // Snap to waypoint to avoid accumulating errors
            self.position = path.waypoint(self.waypoint_index).unwrap();
            self.waypoint_index += 1;
        }

        // If not finished after potentially advancing the waypoint index
        if !path.is_finished(self.waypoint_index) {
            let dir = path.get_direction(self.position, self.waypoint_index);
            self.position += dir * self.speed * dt;
        }
    }

    fn take_damage(&mut self, amount: f32) -> bool {
        self.health -= amount;
        self.is_alive()
    }

    fn get_position(&self) -> Vec2 {
        self.position
    }

    fn get_health(&self) -> f32 {
        self.health
    }

    fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    fn get_reward(&self) -> u32 {
        self.reward
    }

    fn has_reached_end(&self, path: &Path) -> bool {
        path.is_finished(self.waypoint_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn example_path() -> Path {
        Path::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
        ])
    }

    #[test]
    fn test_enemy_initialization() {
        let enemy = BasicEnemy::new(Vec2::ZERO);
        assert_eq!(enemy.get_health(), 100.0);
        assert_eq!(enemy.get_position(), Vec2::ZERO);
        assert!(enemy.is_alive());
        assert_eq!(enemy.reward(), 10);
    }

    #[test]
    fn test_enemy_take_damage() {
        let mut enemy = BasicEnemy::new(Vec2::ZERO);
        assert!(enemy.take_damage(40.0));
        assert_eq!(enemy.get_health(), 60.0);
        assert!(enemy.is_alive());

        assert!(!enemy.take_damage(60.0));
        assert_eq!(enemy.get_health(), 0.0);
        assert!(!enemy.is_alive());
    }

    #[test]
    fn test_enemy_movement() {
        let path = example_path();
        let mut enemy = BasicEnemy::new(path.start().unwrap());

        // Speed is 2.0. In 1 second, it should move 2 units towards (10, 0).
        enemy.update(1.0, &path);
        assert_eq!(enemy.get_position(), Vec2::new(2.0, 0.0));

        // Move another 4 seconds (8 units). Position should be (10.0, 0.0).
        enemy.update(4.0, &path);
        
        // At this exact moment, it's at (10, 0), so it should snap and move on the next update
        enemy.update(0.1, &path);
        // It should have snapped to (10,0) and started moving along Y axis towards (10,10)
        // In 0.1 seconds at speed 2.0, it moves 0.2 units.
        let pos = enemy.get_position();
        assert_eq!(pos.x, 10.0);
        assert!((pos.y - 0.2).abs() < 1e-5);
    }
}
