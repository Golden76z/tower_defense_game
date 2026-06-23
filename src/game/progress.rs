use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProgress {
    pub unlocked: bool,
    pub high_score: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub easy: MapProgress,
    pub medium: MapProgress,
    pub hard: MapProgress,
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
