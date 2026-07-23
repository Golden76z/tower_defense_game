use crate::game::towers::manager::TowerType;
use crate::game::wave_manager::{EnemyType, WaveState};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Current on-disk schema version for the in-game save file. Bump this when the
/// format changes so old files (which default to 0) can be migrated.
pub const SAVE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedTower {
    pub tower_type: TowerType,
    pub position: Vec2,
    pub level: u32,
    pub rotation: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedEnemy {
    pub enemy_type: EnemyType,
    pub position: Vec2,
    pub health: f32,
    pub waypoint_index: usize,
    pub path_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    /// Schema version of this save (see [`SAVE_VERSION`]). Files written before
    /// versioning existed have no field and default to 0.
    #[serde(default)]
    pub version: u32,
    pub map_name: String,
    pub money: i32,
    pub lives: i32,
    pub max_lives: i32,
    pub current_wave_index: usize,
    pub current_spawn_index: usize,
    pub spawn_count_current_group: u32,
    pub spawn_timer: f32,
    pub inter_wave_timer: f32,
    pub wave_state: WaveState,
    pub enemy_spawn_count: usize,
    pub continued_after_victory: bool,
    pub towers: Vec<SavedTower>,
    pub enemies: Vec<SavedEnemy>,
}

impl SaveData {
    /// Saves the data to a JSON file.
    pub fn save_to_file(&self, path: &str) -> Result<(), anyhow::Error> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Loads the data from a JSON file.
    pub fn load_from_file(path: &str) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: Self = serde_json::from_str(&content)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_roundtrip() {
        let temp_file = "isoguard_test_save.json";

        let save_data = SaveData {
            version: SAVE_VERSION,
            map_name: "test_map".to_string(),
            money: 450,
            lives: 4,
            max_lives: 5,
            current_wave_index: 2,
            current_spawn_index: 5,
            spawn_count_current_group: 3,
            spawn_timer: 1.25,
            inter_wave_timer: 0.0,
            wave_state: WaveState::Spawning,
            enemy_spawn_count: 8,
            continued_after_victory: false,
            towers: vec![
                SavedTower {
                    tower_type: TowerType::Basic,
                    position: Vec2::new(1.0, 2.0),
                    level: 2,
                    rotation: std::f32::consts::FRAC_PI_2,
                },
                SavedTower {
                    tower_type: TowerType::Sniper,
                    position: Vec2::new(3.0, 4.0),
                    level: 1,
                    rotation: 0.0,
                },
            ],
            enemies: vec![
                SavedEnemy {
                    enemy_type: EnemyType::Basic,
                    position: Vec2::new(0.5, 0.5),
                    health: 45.0,
                    waypoint_index: 1,
                    path_index: 0,
                },
                SavedEnemy {
                    enemy_type: EnemyType::Fast,
                    position: Vec2::new(2.5, 3.5),
                    health: 12.0,
                    waypoint_index: 4,
                    path_index: 1,
                },
            ],
        };

        // Serialize and save to temp file
        assert!(save_data.save_to_file(temp_file).is_ok());

        // Load back and deserialize
        let loaded_result = SaveData::load_from_file(temp_file);
        assert!(loaded_result.is_ok());
        let loaded = loaded_result.unwrap();

        // Verify values
        assert_eq!(loaded.map_name, save_data.map_name);
        assert_eq!(loaded.money, save_data.money);
        assert_eq!(loaded.lives, save_data.lives);
        assert_eq!(loaded.max_lives, save_data.max_lives);
        assert_eq!(loaded.current_wave_index, save_data.current_wave_index);
        assert_eq!(loaded.towers.len(), save_data.towers.len());
        assert_eq!(loaded.towers[0].level, 2);
        assert_eq!(loaded.enemies.len(), save_data.enemies.len());
        assert_eq!(loaded.enemies[1].health, 12.0);
        assert_eq!(loaded.version, SAVE_VERSION);

        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn pre_version_save_still_loads_with_default_version() {
        // A save written before the `version` field existed must still load,
        // defaulting version to 0 rather than failing to deserialize.
        let legacy = r#"{
            "map_name": "easy", "money": 100, "lives": 3, "max_lives": 5,
            "current_wave_index": 0, "current_spawn_index": 0,
            "spawn_count_current_group": 0, "spawn_timer": 0.0, "inter_wave_timer": 0.0,
            "wave_state": "NotStarted", "enemy_spawn_count": 0,
            "continued_after_victory": false, "towers": [], "enemies": []
        }"#;
        let loaded: SaveData = serde_json::from_str(legacy).unwrap();
        assert_eq!(loaded.version, 0);
        assert_eq!(loaded.map_name, "easy");
    }
}
