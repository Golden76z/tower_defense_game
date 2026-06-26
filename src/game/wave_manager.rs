use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyType {
    Basic,
    Fast,
}

#[derive(Debug, Clone)]
pub struct EnemySpawn {
    pub enemy_type: EnemyType,
    pub count: u32,
    pub spawn_delay: f32,
}

#[derive(Debug, Clone)]
pub struct Wave {
    pub enemy_spawns: Vec<EnemySpawn>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveState {
    NotStarted,
    Spawning,
    WaitingForClean,
    InterWaveDelay,
    CompletedAll,
}


pub struct WaveManager {
    pub waves: Vec<Wave>,
    pub current_wave_index: usize,
    pub current_spawn_index: usize,
    pub spawn_count_current_group: u32,
    pub spawn_timer: f32,
    pub inter_wave_timer: f32,
    pub state: WaveState,
}

impl WaveManager {
    pub fn new(waves: Vec<Wave>) -> Self {
        Self {
            waves,
            current_wave_index: 0,
            current_spawn_index: 0,
            spawn_count_current_group: 0,
            spawn_timer: 0.0,
            inter_wave_timer: 0.0,
            state: WaveState::NotStarted,
        }
    }

    pub fn get_default_waves() -> Vec<Wave> {
        vec![
            // Wave 1: 10 BasicEnemy, 0.5s delay between spawns
            Wave {
                enemy_spawns: vec![EnemySpawn {
                    enemy_type: EnemyType::Basic,
                    count: 10,
                    spawn_delay: 0.5,
                }],
            },
            // Wave 2: 8 BasicEnemy, 5 FastEnemy
            Wave {
                enemy_spawns: vec![
                    EnemySpawn {
                        enemy_type: EnemyType::Basic,
                        count: 8,
                        spawn_delay: 0.5,
                    },
                    EnemySpawn {
                        enemy_type: EnemyType::Fast,
                        count: 5,
                        spawn_delay: 0.3,
                    },
                ],
            },
            // Wave 3: 12 BasicEnemy, 10 FastEnemy
            Wave {
                enemy_spawns: vec![
                    EnemySpawn {
                        enemy_type: EnemyType::Basic,
                        count: 12,
                        spawn_delay: 0.4,
                    },
                    EnemySpawn {
                        enemy_type: EnemyType::Fast,
                        count: 10,
                        spawn_delay: 0.25,
                    },
                ],
            },
        ]
    }

    pub fn start_wave(&mut self, index: usize) -> bool {
        if index < self.waves.len() {
            self.current_wave_index = index;
            self.current_spawn_index = 0;
            self.spawn_count_current_group = 0;
            self.spawn_timer = 0.0;
            self.inter_wave_timer = 0.0;
            self.state = WaveState::Spawning;
            true
        } else {
            false
        }
    }

    pub fn current_wave_number(&self) -> usize {
        self.current_wave_index + 1
    }

    pub fn is_wave_spawning_complete(&self) -> bool {
        if self.current_wave_index >= self.waves.len() {
            return true;
        }
        let wave = &self.waves[self.current_wave_index];
        self.current_spawn_index >= wave.enemy_spawns.len()
    }

    pub fn get_status_message(&self) -> String {
        match self.state {
            WaveState::NotStarted => "Waiting to start".to_string(),
            WaveState::Spawning => format!("Wave {} - Spawning", self.current_wave_number()),
            WaveState::WaitingForClean => format!("Wave {} - In progress", self.current_wave_number()),
            WaveState::InterWaveDelay => format!("Wave {} complete! Next wave in {:.1}s", self.current_wave_number(), self.inter_wave_timer),
            WaveState::CompletedAll => "VICTORY! All waves completed".to_string(),
        }
    }

    pub fn update(&mut self, dt: f32, active_enemy_count: usize) -> Option<EnemyType> {
        match self.state {
            WaveState::NotStarted => None,
            WaveState::Spawning => {
                let wave = &self.waves[self.current_wave_index];
                if self.current_spawn_index >= wave.enemy_spawns.len() {
                    self.state = WaveState::WaitingForClean;
                    return None;
                }

                let group = &wave.enemy_spawns[self.current_spawn_index];
                let mut spawn = None;

                if self.spawn_count_current_group == 0 {
                    self.spawn_count_current_group += 1;
                    self.spawn_timer = 0.0;
                    spawn = Some(group.enemy_type);
                } else {
                    self.spawn_timer += dt;
                    if self.spawn_timer >= group.spawn_delay {
                        self.spawn_timer -= group.spawn_delay;
                        self.spawn_count_current_group += 1;
                        spawn = Some(group.enemy_type);
                    }
                }

                // If we finished this spawn group
                if spawn.is_some() && self.spawn_count_current_group >= group.count {
                    self.current_spawn_index += 1;
                    self.spawn_count_current_group = 0;
                    self.spawn_timer = 0.0;
                }

                // If we finished the entire wave's spawns
                if self.current_spawn_index >= wave.enemy_spawns.len() {
                    self.state = WaveState::WaitingForClean;
                }

                spawn
            }
            WaveState::WaitingForClean => {
                if active_enemy_count == 0 {
                    if self.current_wave_index + 1 < self.waves.len() {
                        self.state = WaveState::InterWaveDelay;
                        self.inter_wave_timer = 10.0;
                    } else {
                        self.state = WaveState::CompletedAll;
                    }
                }
                None
            }
            WaveState::InterWaveDelay => {
                self.inter_wave_timer -= dt;
                if self.inter_wave_timer <= 0.0 {
                    self.current_wave_index += 1;
                    self.current_spawn_index = 0;
                    self.spawn_count_current_group = 0;
                    self.spawn_timer = 0.0;
                    self.state = WaveState::Spawning;
                }
                None
            }
            WaveState::CompletedAll => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_waves_configuration() {
        let waves = WaveManager::get_default_waves();
        assert_eq!(waves.len(), 3);

        // Wave 1
        assert_eq!(waves[0].enemy_spawns.len(), 1);
        assert_eq!(waves[0].enemy_spawns[0].enemy_type, EnemyType::Basic);
        assert_eq!(waves[0].enemy_spawns[0].count, 10);
        assert_eq!(waves[0].enemy_spawns[0].spawn_delay, 0.5);
    }

    #[test]
    fn test_wave_manager_spawning_flow() {
        let waves = vec![
            Wave {
                enemy_spawns: vec![EnemySpawn {
                    enemy_type: EnemyType::Basic,
                    count: 2,
                    spawn_delay: 0.5,
                }],
            },
            Wave {
                enemy_spawns: vec![EnemySpawn {
                    enemy_type: EnemyType::Basic,
                    count: 1,
                    spawn_delay: 0.5,
                }],
            },
        ];

        let mut manager = WaveManager::new(waves);
        assert_eq!(manager.state, WaveState::NotStarted);

        // Start Wave 1
        assert!(manager.start_wave(0));
        assert_eq!(manager.state, WaveState::Spawning);
        assert_eq!(manager.current_wave_number(), 1);

        // Spawn 1 (immediate)
        assert_eq!(manager.update(0.0, 0), Some(EnemyType::Basic));
        assert_eq!(manager.state, WaveState::Spawning);

        // Spawn 2 (delay)
        assert_eq!(manager.update(0.5, 1), Some(EnemyType::Basic));
        // All spawned. Spawning complete, should transition to WaitingForClean
        assert_eq!(manager.state, WaveState::WaitingForClean);

        // Waiting for enemies to die (still active enemies)
        assert_eq!(manager.update(0.5, 2), None);
        assert_eq!(manager.state, WaveState::WaitingForClean);

        // All enemies die -> transition to InterWaveDelay
        assert_eq!(manager.update(0.1, 0), None);
        assert_eq!(manager.state, WaveState::InterWaveDelay);
        assert_eq!(manager.inter_wave_timer, 10.0);

        // Countdown delay (9.9s passes, not started yet)
        assert_eq!(manager.update(9.9, 0), None);
        assert_eq!(manager.state, WaveState::InterWaveDelay);

        // Countdown finishes -> auto-start Wave 2
        assert_eq!(manager.update(0.2, 0), None);
        assert_eq!(manager.state, WaveState::Spawning);
        assert_eq!(manager.current_wave_number(), 2);

        // Wave 2, spawn 1 (immediate)
        assert_eq!(manager.update(0.0, 0), Some(EnemyType::Basic));
        assert_eq!(manager.state, WaveState::WaitingForClean);

        // Wave 2 finishes
        assert_eq!(manager.update(0.1, 0), None);
        assert_eq!(manager.state, WaveState::CompletedAll);
    }
}
