use glam::Vec2;
use crate::game::map::path::Path;
use crate::game::enemies::enemy_base::Enemy;

/// A faster but weaker enemy type with path following and lower health.
pub struct FastEnemy {
    position: Vec2,
    health: f32,
    speed: f32,
    reward: u32,
    waypoint_index: usize,
    path: Option<Path>,
}

impl FastEnemy {
    /// Creates a new fast enemy at the given starting position.
    pub fn new(start_position: Vec2) -> Self {
        Self {
            position: start_position,
            health: 50.0,
            speed: 4.0,
            reward: 5,
            waypoint_index: 1, // Start by moving to the second waypoint
            path: None,
        }
    }

    /// Builder to set a specific path for the enemy.
    pub fn with_path(mut self, path: Path) -> Self {
        self.path = Some(path);
        self
    }

    /// Gets the gold reward for defeating this enemy.
    pub fn reward(&self) -> u32 {
        self.reward
    }
}

impl Enemy for FastEnemy {
    fn update(&mut self, dt: f32, path: &Path) {
        let p = self.path.as_ref().unwrap_or(path);
        if p.is_finished(self.waypoint_index) {
            return;
        }

        // Check if we reached the current waypoint
        if p.reached_waypoint(self.position, self.waypoint_index) {
            // Snap to waypoint to avoid accumulating errors
            self.position = p.waypoint(self.waypoint_index).unwrap();
            self.waypoint_index += 1;
        }

        // If not finished after potentially advancing the waypoint index
        if !p.is_finished(self.waypoint_index) {
            let dir = p.get_direction(self.position, self.waypoint_index);
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
        let p = self.path.as_ref().unwrap_or(path);
        p.is_finished(self.waypoint_index)
    }

    fn enemy_type(&self) -> crate::game::wave_manager::EnemyType {
        crate::game::wave_manager::EnemyType::Fast
    }

    fn get_max_health(&self) -> f32 {
        50.0
    }

    fn get_color(&self) -> [f32; 4] {
        [0.2, 0.4, 1.0, 1.0] // Blue
    }

    fn get_scale(&self) -> f32 {
        0.035 // Smaller than BasicEnemy (0.05)
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
    fn test_fast_enemy_initialization() {
        let enemy = FastEnemy::new(Vec2::ZERO);
        assert_eq!(enemy.get_health(), 50.0);
        assert_eq!(enemy.get_max_health(), 50.0);
        assert_eq!(enemy.get_position(), Vec2::ZERO);
        assert!(enemy.is_alive());
        assert_eq!(enemy.reward(), 5);
        assert_eq!(enemy.get_scale(), 0.035);
        assert_eq!(enemy.get_color(), [0.2, 0.4, 1.0, 1.0]);
    }

    #[test]
    fn test_fast_enemy_take_damage() {
        let mut enemy = FastEnemy::new(Vec2::ZERO);
        assert!(enemy.take_damage(20.0));
        assert_eq!(enemy.get_health(), 30.0);
        assert!(enemy.is_alive());

        assert!(!enemy.take_damage(30.0));
        assert_eq!(enemy.get_health(), 0.0);
        assert!(!enemy.is_alive());
    }

    #[test]
    fn test_fast_enemy_movement() {
        let path = example_path();
        let mut enemy = FastEnemy::new(path.start().unwrap());

        // Speed is 4.0. In 1 second, it should move 4 units towards (10, 0).
        enemy.update(1.0, &path);
        assert_eq!(enemy.get_position(), Vec2::new(4.0, 0.0));

        // Move another 1.5 seconds (6 units). Position should be (10.0, 0.0).
        enemy.update(1.5, &path);
        
        // At this exact moment, it's at (10, 0), so it should snap and move on the next update
        enemy.update(0.1, &path);
        // It should have snapped to (10,0) and started moving along Y axis towards (10,10)
        // In 0.1 seconds at speed 4.0, it moves 0.4 units.
        let pos = enemy.get_position();
        assert_eq!(pos.x, 10.0);
        assert!((pos.y - 0.4).abs() < 1e-5);
    }
}
