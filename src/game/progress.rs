use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProgress {
    pub unlocked: bool,
    pub high_score: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Achievements {
    pub first_blood: bool,
    pub architect: bool,
    pub wave_survivor: bool,
    pub gold_miner: bool,
    pub victorious: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameStatistics {
    pub games_played: u32,
    pub wins: u32,
    pub waves_completed: u32,
    pub enemies_killed: u32,
    pub towers_placed: u32,
    pub high_score: u32,
    #[serde(default)]
    pub gold_spent: u32,
    #[serde(default)]
    pub achievements: Achievements,
}

impl Default for GameStatistics {
    fn default() -> Self {
        Self {
            games_played: 0,
            wins: 0,
            waves_completed: 0,
            enemies_killed: 0,
            towers_placed: 0,
            high_score: 0,
            gold_spent: 0,
            achievements: Achievements::default(),
        }
    }
}

impl GameStatistics {
    pub fn check_achievements(&mut self) {
        if self.enemies_killed >= 1 {
            self.achievements.first_blood = true;
        }
        if self.towers_placed >= 20 {
            self.achievements.architect = true;
        }
        if self.waves_completed >= 10 {
            self.achievements.wave_survivor = true;
        }
        if self.gold_spent >= 5000 {
            self.achievements.gold_miner = true;
        }
        if self.wins >= 1 {
            self.achievements.victorious = true;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub easy: MapProgress,
    pub medium: MapProgress,
    pub hard: MapProgress,
    #[serde(default)]
    pub statistics: GameStatistics,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            easy: MapProgress {
                unlocked: true,
                high_score: 0,
                completed: false,
            },
            medium: MapProgress {
                unlocked: false,
                high_score: 0,
                completed: false,
            },
            hard: MapProgress {
                unlocked: false,
                high_score: 0,
                completed: false,
            },
            statistics: GameStatistics::default(),
        }
    }
}

impl SaveData {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("isoguard_progress.json") {
            if let Ok(data) = serde_json::from_str::<SaveData>(&content) {
                return data;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("isoguard_progress.json", content);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backward_compatibility() {
        // JSON content without the statistics field (simulating an older progress file)
        let legacy_json = r#"{
            "easy": { "unlocked": true, "high_score": 1500, "completed": true },
            "medium": { "unlocked": true, "high_score": 0, "completed": false },
            "hard": { "unlocked": false, "high_score": 0, "completed": false }
        }"#;

        let parsed: SaveData = serde_json::from_str(legacy_json).unwrap();
        
        // Easy progress should be loaded correctly
        assert!(parsed.easy.unlocked);
        assert_eq!(parsed.easy.high_score, 1500);
        assert!(parsed.easy.completed);
        
        // The statistics field should automatically fall back to its Default implementation
        assert_eq!(parsed.statistics, GameStatistics::default());
        assert_eq!(parsed.statistics.games_played, 0);
        assert_eq!(parsed.statistics.enemies_killed, 0);
    }

    #[test]
    fn test_full_serialization_roundtrip() {
        let mut save_data = SaveData::default();
        save_data.statistics.games_played = 5;
        save_data.statistics.enemies_killed = 42;
        save_data.statistics.wins = 2;

        let serialized = serde_json::to_string(&save_data).unwrap();
        let deserialized: SaveData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.statistics.games_played, 5);
        assert_eq!(deserialized.statistics.enemies_killed, 42);
        assert_eq!(deserialized.statistics.wins, 2);
    }

    #[test]
    fn test_achievements_unlocking() {
        let mut save_data = SaveData::default();
        
        // At start, no achievements should be unlocked
        assert!(!save_data.statistics.achievements.first_blood);
        assert!(!save_data.statistics.achievements.architect);
        assert!(!save_data.statistics.achievements.wave_survivor);
        assert!(!save_data.statistics.achievements.gold_miner);
        assert!(!save_data.statistics.achievements.victorious);

        // Test First Blood unlock
        save_data.statistics.enemies_killed = 1;
        save_data.statistics.check_achievements();
        assert!(save_data.statistics.achievements.first_blood);

        // Test Architect unlock
        save_data.statistics.towers_placed = 20;
        save_data.statistics.check_achievements();
        assert!(save_data.statistics.achievements.architect);

        // Test Wave Survivor unlock
        save_data.statistics.waves_completed = 10;
        save_data.statistics.check_achievements();
        assert!(save_data.statistics.achievements.wave_survivor);

        // Test Gold Miner unlock
        save_data.statistics.gold_spent = 5000;
        save_data.statistics.check_achievements();
        assert!(save_data.statistics.achievements.gold_miner);

        // Test Victorious unlock
        save_data.statistics.wins = 1;
        save_data.statistics.check_achievements();
        assert!(save_data.statistics.achievements.victorious);
    }
}
