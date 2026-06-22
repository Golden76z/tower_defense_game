use glam::Vec2;
use crate::game::enemies::enemy_base::Enemy;
use crate::game::projectiles::Projectile;
use crate::game::towers::tower_base::Tower;

pub struct BasicTower {
    pub position: Vec2,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32, // shots per second
    pub cooldown: f32,
    pub cost: u32,
    pub rotation: f32,
    pub level: u32,
}

impl BasicTower {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            range: 5.0,
            damage: 25.0,
            fire_rate: 1.0,
            cooldown: 0.0,
            cost: 50,
            rotation: 0.0,
            level: 1,
        }
    }

    pub fn get_current_range(&self) -> f32 {
        match self.level {
            1 => self.range,
            2 => self.range * 1.25,
            3 => self.range * 1.5,
            _ => self.range,
        }
    }

    pub fn get_current_damage(&self) -> f32 {
        match self.level {
            1 => self.damage,
            2 => self.damage * 1.5,
            3 => self.damage * 2.0,
            _ => self.damage,
        }
    }
}

impl Tower for BasicTower {
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
                if dist <= self.get_range() && dist < min_distance {
                    min_distance = dist;
                    nearest_enemy = Some(enemy);
                }
            }

            if let Some(target) = nearest_enemy {
                let direction = target.get_position() - self.position;
                self.rotation = direction.y.atan2(direction.x);
                self.cooldown = 1.0 / self.fire_rate;
                // 0.6 = barrel height above the tile surface in tile units
                // (tower model barrel at y~1.0 * tower render scale 0.6).
                return Some(Projectile::new(self.position, target.get_position(), 15.0, self.get_damage(), 0.6));
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
        self.get_current_range()
    }

    fn get_rotation(&self) -> f32 {
        self.rotation
    }

    fn tower_type(&self) -> crate::game::towers::manager::TowerType {
        crate::game::towers::manager::TowerType::Basic
    }

    fn get_level(&self) -> u32 {
        self.level
    }

    fn upgrade(&mut self) -> Result<(), &'static str> {
        if self.level < 3 {
            self.level += 1;
            Ok(())
        } else {
            Err("Max level reached")
        }
    }

    fn get_upgrade_cost(&self) -> Option<i32> {
        match self.level {
            1 => Some((self.cost / 2) as i32), // 50%
            2 => Some(self.cost as i32),        // 100%
            _ => None,
        }
    }

    fn get_damage(&self) -> f32 {
        self.get_current_damage()
    }

    fn get_fire_rate(&self) -> f32 {
        self.fire_rate
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
    fn test_tower_shoots_enemy_in_range() {
        let mut tower = BasicTower::new(Vec2::new(0.0, 0.0));
        let enemies: Vec<Box<dyn Enemy>> = vec![
            Box::new(MockEnemy::new(Vec2::new(3.0, 4.0), 100.0)), // Distance 5.0
        ];
        
        let projectile = tower.update(0.1, &enemies);
        assert!(projectile.is_some(), "Tower should shoot at enemy in range");
        assert_eq!(tower.cooldown, 1.0); // Assuming fire_rate is 1.0
    }

    #[test]
    fn test_tower_doesnt_shoot_enemy_out_of_range() {
        let mut tower = BasicTower::new(Vec2::new(0.0, 0.0));
        let enemies: Vec<Box<dyn Enemy>> = vec![
            Box::new(MockEnemy::new(Vec2::new(4.0, 4.0), 100.0)), // Distance ~5.65 > 5.0
        ];
        
        let projectile = tower.update(0.1, &enemies);
        assert!(projectile.is_none(), "Tower should not shoot at enemy out of range");
        assert_eq!(tower.cooldown, 0.0);
    }

    #[test]
    fn test_cooldown_prevents_rapid_fire() {
        let mut tower = BasicTower::new(Vec2::new(0.0, 0.0));
        let enemies: Vec<Box<dyn Enemy>> = vec![
            Box::new(MockEnemy::new(Vec2::new(2.0, 0.0), 100.0)),
        ];
        
        // First shot
        let projectile1 = tower.update(0.1, &enemies);
        assert!(projectile1.is_some());
        
        // Immediate second update (no time passed effectively for cooldown)
        let projectile2 = tower.update(0.1, &enemies);
        assert!(projectile2.is_none(), "Tower should not shoot while on cooldown");
        assert!(tower.cooldown > 0.0);
    }
    
    #[test]
    fn test_tower_rotation() {
        let mut tower = BasicTower::new(Vec2::new(0.0, 0.0));
        let enemies: Vec<Box<dyn Enemy>> = vec![
            Box::new(MockEnemy::new(Vec2::new(0.0, 5.0), 100.0)),
        ];
        
        tower.update(0.1, &enemies);
        
        // atan2(5.0, 0.0) is PI/2
        use std::f32::consts::PI;
        assert!((tower.rotation - PI / 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_basic_tower_upgrades() {
        let mut tower = BasicTower::new(Vec2::new(0.0, 0.0));
        assert_eq!(tower.get_level(), 1);
        assert_eq!(tower.get_upgrade_cost(), Some(25));
        assert_eq!(tower.get_damage(), 25.0);
        assert_eq!(tower.get_range(), 5.0);

        // Upgrade to lvl 2
        assert!(tower.upgrade().is_ok());
        assert_eq!(tower.get_level(), 2);
        assert_eq!(tower.get_upgrade_cost(), Some(50));
        assert_eq!(tower.get_damage(), 37.5);
        assert_eq!(tower.get_range(), 6.25);

        // Upgrade to lvl 3
        assert!(tower.upgrade().is_ok());
        assert_eq!(tower.get_level(), 3);
        assert_eq!(tower.get_upgrade_cost(), None);
        assert_eq!(tower.get_damage(), 50.0);
        assert_eq!(tower.get_range(), 7.5);

        // Attempting to upgrade further should fail
        assert!(tower.upgrade().is_err());
        assert_eq!(tower.get_level(), 3);
    }
}
