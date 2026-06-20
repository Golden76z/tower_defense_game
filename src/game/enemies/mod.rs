pub mod enemy_base;
pub mod basic_enemy;

use crate::game::map::path::Path;
use enemy_base::Enemy;

#[derive(Default)]
pub struct EnemyManager {
    pub(crate) enemies: Vec<Box<dyn Enemy>>,
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
        }
    }

    pub fn spawn_enemy(&mut self, enemy: Box<dyn Enemy>) {
        self.enemies.push(enemy);
    }

    pub fn update(&mut self, dt: f32, path: &Path) {
        // Update all enemies
        for enemy in &mut self.enemies {
            enemy.update(dt, path);
        }

        // Remove dead enemies
        self.enemies.retain(|enemy| enemy.is_alive());
    }

    pub fn get_enemies(&self) -> &[Box<dyn Enemy>] {
        &self.enemies
    }

    pub fn count(&self) -> usize {
        self.enemies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::enemies::basic_enemy::BasicEnemy;
    use glam::Vec2;

    #[test]
    fn test_enemy_manager() {
        let mut manager = EnemyManager::new();
        let path = Path::new(vec![Vec2::ZERO, Vec2::new(10.0, 0.0)]);

        // Spawn 10 enemies
        for _ in 0..10 {
            manager.spawn_enemy(Box::new(BasicEnemy::new(Vec2::ZERO)));
        }

        assert_eq!(manager.count(), 10);

        // Kill 5 enemies
        for i in 0..5 {
            manager.enemies[i].take_damage(100.0);
        }

        // Update to remove dead enemies
        manager.update(0.1, &path);

        // Verify 5 remain
        assert_eq!(manager.count(), 5);
        assert_eq!(manager.get_enemies().len(), 5);
    }
}
