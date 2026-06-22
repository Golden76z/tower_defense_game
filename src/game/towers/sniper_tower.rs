use glam::Vec2;
use crate::game::enemies::enemy_base::Enemy;
use crate::game::projectiles::Projectile;
use crate::game::towers::tower_base::Tower;
use crate::game::towers::manager::TowerType;

pub struct SniperTower {
    pub position: Vec2,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32, // shots per second
    pub cooldown: f32,
    pub cost: u32,
    pub rotation: f32,
}

impl SniperTower {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            range: 12.0,
            damage: 100.0,
            fire_rate: 0.3, // 0.3 shots per second
            cooldown: 0.0,
            cost: 150,
            rotation: 0.0,
        }
    }
}

impl Tower for SniperTower {
    fn update(&mut self, dt: f32, enemies: &[Box<dyn Enemy>]) -> Option<Projectile> {
        if self.cooldown > 0.0 {
            self.cooldown -= dt;
        }

        if self.can_shoot() {
            let mut nearest_enemy: Option<&Box<dyn Enemy>> = None;
            let mut min_distance = f32::MAX;

            for enemy in enemies {
                if !enemy.is_alive() {
                    continue;
                }
                
                let dist = self.position.distance(enemy.get_position());
                if dist <= self.range && dist < min_distance {
                    min_distance = dist;
                    nearest_enemy = Some(enemy);
                }
            }

            if let Some(target) = nearest_enemy {
                let direction = target.get_position() - self.position;
                self.rotation = direction.y.atan2(direction.x);
                self.cooldown = 1.0 / self.fire_rate;
                // 0.75 = barrel height above the tile surface in tile units
                // (tower model barrel at y~1.25 * tower render scale 0.6).
                // Use a high speed of 25.0 for sniper shots.
                return Some(Projectile::new(self.position, target.get_position(), 25.0, self.damage, 0.75));
            }
        }

        None
    }

    fn can_shoot(&self) -> bool {
        self.cooldown <= 0.0
    }

    fn get_position(&self) -> Vec2 {
        self.position
    }

    fn get_range(&self) -> f32 {
        self.range
    }

    fn get_rotation(&self) -> f32 {
        self.rotation
    }

    fn tower_type(&self) -> TowerType {
        TowerType::Sniper
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::path::Path;

    struct MockEnemy {
        position: Vec2,
        health: f32,
    }

    impl MockEnemy {
        fn new(position: Vec2, health: f32) -> Self {
            Self { position, health }
        }
    }

    impl Enemy for MockEnemy {
        fn update(&mut self, _dt: f32, _path: &Path) {}
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
            0
        }
        fn has_reached_end(&self, _path: &Path) -> bool {
            false
        }
        fn enemy_type(&self) -> crate::game::wave_manager::EnemyType {
            crate::game::wave_manager::EnemyType::Basic
        }
        fn get_max_health(&self) -> f32 {
            100.0
        }
        fn get_color(&self) -> [f32; 4] {
            [1.0, 0.2, 0.2, 1.0]
        }
        fn get_scale(&self) -> f32 {
            0.05
        }
    }

    #[test]
    fn test_sniper_tower_initialization() {
        let tower = SniperTower::new(Vec2::new(1.0, 2.0));
        assert_eq!(tower.get_position(), Vec2::new(1.0, 2.0));
        assert_eq!(tower.get_range(), 12.0);
        assert_eq!(tower.damage, 100.0);
        assert_eq!(tower.fire_rate, 0.3);
        assert_eq!(tower.tower_type(), TowerType::Sniper);
    }

    #[test]
    fn test_sniper_tower_shoots_enemy_in_range() {
        let mut tower = SniperTower::new(Vec2::new(0.0, 0.0));
        let enemies: Vec<Box<dyn Enemy>> = vec![
            Box::new(MockEnemy::new(Vec2::new(6.0, 6.0), 100.0)), // Distance ~8.48 < 12.0
        ];
        
        let projectile = tower.update(0.1, &enemies);
        assert!(projectile.is_some(), "Sniper should shoot at enemy in range");
        assert_eq!(tower.cooldown, 1.0 / 0.3); // fire_rate is 0.3
    }

    #[test]
    fn test_sniper_tower_doesnt_shoot_enemy_out_of_range() {
        let mut tower = SniperTower::new(Vec2::new(0.0, 0.0));
        let enemies: Vec<Box<dyn Enemy>> = vec![
            Box::new(MockEnemy::new(Vec2::new(10.0, 10.0), 100.0)), // Distance ~14.14 > 12.0
        ];
        
        let projectile = tower.update(0.1, &enemies);
        assert!(projectile.is_none(), "Sniper should not shoot at enemy out of range");
        assert_eq!(tower.cooldown, 0.0);
    }
}
