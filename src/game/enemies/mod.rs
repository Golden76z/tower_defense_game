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

    pub fn update(&mut self, dt: f32, path: &Path) -> Vec<(glam::Vec2, u32)> {
        // Update all enemies
        for enemy in &mut self.enemies {
            enemy.update(dt, path);
        }

        // Identify, collect, and remove dead enemies
        let mut killed = Vec::new();
        let mut alive = Vec::new();
        for enemy in std::mem::take(&mut self.enemies) {
            if enemy.is_alive() {
                alive.push(enemy);
            } else {
                killed.push((enemy.get_position(), enemy.get_reward()));
            }
        }
        self.enemies = alive;

        killed
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

    #[test]
    fn test_enemy_manager_rewards_on_kill() {
        let mut manager = EnemyManager::new();
        let path = Path::new(vec![Vec2::ZERO, Vec2::new(10.0, 0.0)]);

        // Spawn 3 basic enemies
        manager.spawn_enemy(Box::new(BasicEnemy::new(Vec2::ZERO)));
        manager.spawn_enemy(Box::new(BasicEnemy::new(Vec2::ZERO)));
        manager.spawn_enemy(Box::new(BasicEnemy::new(Vec2::ZERO)));

        // Kill 2 of them
        manager.enemies[0].take_damage(100.0);
        manager.enemies[2].take_damage(100.0);

        // Update
        let killed = manager.update(0.1, &path);

        // Should return 2 rewards, each of 10 gold
        assert_eq!(killed.len(), 2);
        assert_eq!(killed[0].1, 10);
        assert_eq!(killed[1].1, 10);

        // 1 enemy remains alive
        assert_eq!(manager.count(), 1);
    }
}
