use serde::{Deserialize, Serialize};

/// Current on-disk schema version for the progression save file. Bump this when
/// the format changes so old files (which default to 0) can be migrated.
pub const SAVE_VERSION: u32 = 1;

/// Path of the progression save file, relative to the working directory.
const PROGRESS_PATH: &str = "isoguard_progress.json";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    /// Schema version of this save (see [`SAVE_VERSION`]). Files written before
    /// versioning existed have no field and default to 0.
    #[serde(default)]
    pub version: u32,
    pub easy: MapProgress,
    pub medium: MapProgress,
    pub hard: MapProgress,
    #[serde(default)]
    pub statistics: GameStatistics,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
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
        Self::load_from_path(PROGRESS_PATH)
    }

    /// Loads progress from `path`, distinguishing a missing file (first run →
    /// defaults) from a corrupt one (back it up, then defaults) so a parse error
    /// never silently wipes real progress.
    fn load_from_path(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Err(_) => Self::default(),
            Ok(content) => match serde_json::from_str::<SaveData>(&content) {
                Ok(data) => data,
                Err(e) => {
                    let backup = format!("{path}.bak");
                    log::error!(
                        "Progress file '{path}' is corrupt ({e}); backing up to '{backup}' and starting fresh"
                    );
                    let _ = std::fs::rename(path, &backup);
                    Self::default()
                }
            },
        }
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(PROGRESS_PATH, content);
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

    #[test]
    fn save_data_records_version_and_legacy_files_still_load() {
        // New saves carry the current schema version.
        assert_eq!(SaveData::default().version, SAVE_VERSION);
        let json = serde_json::to_string(&SaveData::default()).unwrap();
        assert!(json.contains("\"version\""));

        // Pre-versioning files (no version field) still load, defaulting to 0.
        let legacy = r#"{
            "easy": { "unlocked": true, "high_score": 1500, "completed": true },
            "medium": { "unlocked": false, "high_score": 0, "completed": false },
            "hard": { "unlocked": false, "high_score": 0, "completed": false }
        }"#;
        let parsed: SaveData = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.version, 0, "pre-version files default to version 0");
        assert!(parsed.easy.unlocked);
    }

    #[test]
    fn corrupt_progress_file_is_backed_up_not_silently_overwritten() {
        // Regression: any parse error used to return Default silently, and the
        // next save() overwrote the file — permanently wiping real progress.
        let path = std::env::temp_dir().join("isoguard_progress_corrupt_test.json");
        let path_str = path.to_str().unwrap();
        let backup = format!("{path_str}.bak");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);

        std::fs::write(&path, b"{ not valid json ").unwrap();
        let data = SaveData::load_from_path(path_str);

        // Falls back to defaults...
        assert!(data.easy.unlocked);
        // ...but the corrupt original is preserved as a .bak, not overwritten.
        assert!(
            std::fs::metadata(&backup).is_ok(),
            "corrupt file should be backed up"
        );
        assert!(
            std::fs::metadata(&path).is_err(),
            "corrupt file should be moved aside"
        );

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);
    }

    #[test]
    fn missing_progress_file_loads_defaults_without_a_backup() {
        let path = std::env::temp_dir().join("isoguard_progress_missing_test.json");
        let path_str = path.to_str().unwrap();
        let backup = format!("{path_str}.bak");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);

        let data = SaveData::load_from_path(path_str);
        assert!(data.easy.unlocked);
        assert!(
            std::fs::metadata(&backup).is_err(),
            "a missing file is a first run, not a corruption — no backup"
        );
    }
}
