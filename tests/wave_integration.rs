use tower_defense_lib::game::wave_manager::{WaveManager, Wave, EnemySpawn, EnemyType, WaveState};

#[test]
fn test_wave_integration_flow() {
    let waves = vec![
        Wave {
            enemy_spawns: vec![EnemySpawn {
                enemy_type: EnemyType::Basic,
                count: 3,
                spawn_delay: 1.0,
            }],
        },
        Wave {
            enemy_spawns: vec![EnemySpawn {
                enemy_type: EnemyType::Basic,
                count: 1,
                spawn_delay: 1.0,
            }],
        },
    ];

    let mut wm = WaveManager::new(waves);
    assert_eq!(wm.state, WaveState::NotStarted);
    assert_eq!(wm.current_wave_number(), 1);

    // 1. Start Wave 1
    assert!(wm.start_wave(0));
    assert_eq!(wm.state, WaveState::Spawning);

    // 2. Spawn 1st enemy (immediate)
    let s1 = wm.update(0.0, 0);
    assert_eq!(s1, Some(EnemyType::Basic));
    assert_eq!(wm.state, WaveState::Spawning);

    // Update with 0.5s (no spawn)
    assert_eq!(wm.update(0.5, 1), None);

    // 3. Spawn 2nd enemy at 1.0s total elapsed in wave
    let s2 = wm.update(0.5, 1);
    assert_eq!(s2, Some(EnemyType::Basic));
    assert_eq!(wm.state, WaveState::Spawning);

    // 4. Spawn 3rd enemy at 2.0s total elapsed
    let s3 = wm.update(1.0, 2);
    assert_eq!(s3, Some(EnemyType::Basic));
    // Spawning should be finished and transitioned to WaitingForClean
    assert_eq!(wm.state, WaveState::WaitingForClean);

    // 5. Keep waiting while enemies are alive
    assert_eq!(wm.update(1.0, 3), None);
    assert_eq!(wm.state, WaveState::WaitingForClean);

    // 6. All enemies are dead -> transition to InterWaveDelay
    assert_eq!(wm.update(0.1, 0), None);
    assert_eq!(wm.state, WaveState::InterWaveDelay);
    assert_eq!(wm.inter_wave_timer, 10.0);

    // 7. Tick delay timer
    assert_eq!(wm.update(5.0, 0), None);
    assert_eq!(wm.state, WaveState::InterWaveDelay);
    assert!(wm.inter_wave_timer > 0.0);

    // 8. Countdown finishes -> Wave 2 starts automatically
    assert_eq!(wm.update(5.0, 0), None);
    assert_eq!(wm.state, WaveState::Spawning);
    assert_eq!(wm.current_wave_number(), 2);

    // 9. Wave 2: Spawn 1st enemy (immediate)
    let s4 = wm.update(0.0, 0);
    assert_eq!(s4, Some(EnemyType::Basic));
    assert_eq!(wm.state, WaveState::WaitingForClean);

    // 10. All dead -> CompletedAll (no more waves left)
    assert_eq!(wm.update(0.0, 0), None);
    assert_eq!(wm.state, WaveState::CompletedAll);
    assert_eq!(wm.get_status_message(), "VICTORY! All waves completed");
}
