use glam::Vec2;
use crate::game::towers::tower_base::Tower;
use crate::game::towers::basic_tower::BasicTower;
use crate::game::projectiles::Projectile;
use crate::game::enemies::enemy_base::Enemy;
use crate::game::map::map::Map;
use crate::game::map::tile::TileType;
use crate::game::economy::Economy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TowerType {
    Basic,
}

impl TowerType {
    pub fn cost(&self) -> i32 {
        match self {
            TowerType::Basic => 50,
        }
    }
}

pub struct TowerManager {
    pub towers: Vec<Box<dyn Tower>>,
}

impl TowerManager {
    pub fn new() -> Self {
        Self {
            towers: Vec::new(),
        }
    }

    pub fn can_place_tower(&self, map: &Map, position: Vec2, tower_type: TowerType, economy: &Economy) -> Result<(), &'static str> {
        let cost = tower_type.cost();
        if !economy.can_afford(cost) {
            return Err("Insufficient funds");
        }

        let grid_x = position.x.round() as i32;
        let grid_y = position.y.round() as i32;

        let tile = map.get_tile(grid_x, grid_y).ok_or("Out of bounds")?;

        if tile.tile_type == TileType::Path {
            return Err("Cannot place on path tiles");
        }

        // Check for existing towers
        for tower in &self.towers {
            let tower_pos = tower.get_position();
            let t_grid_x = tower_pos.x.round() as i32;
            let t_grid_y = tower_pos.y.round() as i32;
            if grid_x == t_grid_x && grid_y == t_grid_y {
                return Err("Tower already exists at this location");
            }
        }
        
        Ok(())
    }

    pub fn place_tower(&mut self, map: &Map, position: Vec2, tower_type: TowerType, economy: &mut Economy) -> Result<(), &'static str> {
        self.can_place_tower(map, position, tower_type, economy)?;

        let cost = tower_type.cost();
        if !economy.purchase(cost) {
            return Err("Insufficient funds");
        }

        let tower: Box<dyn Tower> = match tower_type {
            TowerType::Basic => Box::new(BasicTower::new(position)),
        };

        self.towers.push(tower);
        Ok(())
    }

    pub fn update_all(&mut self, dt: f32, enemies: &[Box<dyn Enemy>]) -> Vec<Projectile> {
        let mut projectiles = Vec::new();

        for tower in &mut self.towers {
            if let Some(projectile) = tower.update(dt, enemies) {
                projectiles.push(projectile);
            }
        }

        projectiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::tile::Tile;

    // Helper for testing
    fn setup_map() -> Map {
        let mut map = Map::new(5, 5);
        // Put a path at 2, 2
        let path_tile = Tile::new(TileType::Path, 2, 0, 2, true);
        map.set_tile(2, 2, path_tile).unwrap();
        map
    }

    #[test]
    fn test_place_valid_tower() {
        let mut manager = TowerManager::new();
        let map = setup_map();
        let position = Vec2::new(1.0, 1.0);
        let mut economy = Economy::new(100);
        
        let result = manager.place_tower(&map, position, TowerType::Basic, &mut economy);
        assert!(result.is_ok());
        assert_eq!(manager.towers.len(), 1);
    }

    #[test]
    fn test_place_tower_on_path() {
        let mut manager = TowerManager::new();
        let map = setup_map();
        let position = Vec2::new(2.0, 2.0); // Path is at 2,2
        let mut economy = Economy::new(100);
        
        let result = manager.place_tower(&map, position, TowerType::Basic, &mut economy);
        assert!(result.is_err());
        assert_eq!(manager.towers.len(), 0);
    }

    #[test]
    fn test_place_tower_out_of_bounds() {
        let mut manager = TowerManager::new();
        let map = setup_map();
        let position = Vec2::new(10.0, 10.0);
        let mut economy = Economy::new(100);
        
        let result = manager.place_tower(&map, position, TowerType::Basic, &mut economy);
        assert!(result.is_err());
        assert_eq!(manager.towers.len(), 0);
    }

    #[test]
    fn test_place_tower_overlap() {
        let mut manager = TowerManager::new();
        let map = setup_map();
        let position = Vec2::new(1.0, 1.0);
        let mut economy = Economy::new(100);
        
        assert!(manager.place_tower(&map, position, TowerType::Basic, &mut economy).is_ok());
        assert!(manager.place_tower(&map, position, TowerType::Basic, &mut economy).is_err());
        assert_eq!(manager.towers.len(), 1);
    }

    #[test]
    fn test_place_tower_insufficient_funds() {
        let mut manager = TowerManager::new();
        let map = setup_map();
        let position = Vec2::new(1.0, 1.0);
        let mut economy = Economy::new(30); // Basic tower costs 50
        
        let result = manager.place_tower(&map, position, TowerType::Basic, &mut economy);
        assert_eq!(result, Err("Insufficient funds"));
        assert_eq!(manager.towers.len(), 0);
        assert_eq!(economy.money, 30); // Money not deducted
    }

    #[test]
    fn test_place_tower_deducts_money() {
        let mut manager = TowerManager::new();
        let map = setup_map();
        let position = Vec2::new(1.0, 1.0);
        let mut economy = Economy::new(100);
        
        let result = manager.place_tower(&map, position, TowerType::Basic, &mut economy);
        assert!(result.is_ok());
        assert_eq!(economy.money, 50); // 100 - 50 = 50
    }
}
