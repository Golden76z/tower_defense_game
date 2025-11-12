# Tower Defense Game

A 2D isometric tower defense game built from scratch in Rust with a custom game engine.

## About

Defend against waves of enemies by strategically placing towers, building walls, and upgrading your arsenal. Features a progression system with card-based upgrades, multiple biomes, boss battles, and an AI versus mode.

**Status:** 🚧 In Development (Phase 0 - Foundation)

## Features (Planned)

- **Core Tower Defense**: Classic TD gameplay with multiple tower types
- **Building System**: Place towers and walls with rotation support
- **Advanced Combat**: AOE damage, projectile reflection, multiple targeting modes
- **Progression System**: XP levels with card-based upgrade choices (Vampire Survivors style)
- **Campaign Mode**: 50+ stages across 5 unique biomes with story elements
- **Boss Battles**: Epic encounters at the end of each biome
- **Versus AI**: 1v1 Age of War style battles against AI opponent
- **Multiple Game Modes**: Endless mode, daily challenges, achievements

## Built With

- **Language**: Rust 2021 Edition
- **Graphics**: wgpu (modern, cross-platform rendering)
- **Windowing**: winit
- **Math**: glam
- **UI**: egui (planned)

## Quick Start

### Prerequisites

- Rust 1.70+ ([Install](https://rustup.rs/))
- Git

### Building

```bash
# Clone the repository - HTTPS
git clone https://github.com/Golden76z/tower_defense_game.git
# SSH
git clone git@github.com:Golden76z/tower_defense_game.git
cd tower_defense_game

# Build and run
cargo run
```
<!---->
<!-- ### Development Build -->
<!---->
<!-- ```bash -->
<!-- cargo build -->
<!-- ``` -->
<!---->
<!-- ### Release Build -->
<!---->
<!-- ```bash -->
<!-- cargo build --release -->
<!-- ``` -->
<!---->
## Testing

```bash
cargo test                      # Run all tests
cargo test --lib               # Unit tests only
cargo test --test '*'          # Integration tests only
cargo test --doc               # Doc tests only
cargo test test_name           # Specific test
cargo test -- --nocapture      # Show println! output
cargo test -- --test-threads=1 # Run serially
```

### Coverage
```bash
cargo tarpaulin --out Html
```

### Benchmarking

```bash
cargo bench
cargo bench bench_name
```

### Linting & Formatting
```bash
cargo fmt
cargo clippy
cargo clippy --fix
```

### Scripts
```bash
bash scripts/run_tests.sh         # Full test suite
```

## Project Structure

```
tower_defense_game/
├── src/
│   ├── renderer/       # Graphics engine
│   ├── engine/         # Core systems (input, time)
│   ├── game/           # Game logic (towers, enemies, map)
│   └── math/           # Math utilities (isometric conversion)
├── assets/
│   ├── sprites/        # Game art
│   ├── sounds/         # Audio files
│   └── maps/           # Level data
├── tests/              # Integration tests
└── docs/               # Documentation
```

## License

This project is dual-licensed under:

- MIT License ([LICENSE](LICENSE))

## Contact

- **Issues**: [GitHub Issues](https://github.com/yourusername/tower_defense_game/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/tower_defense_game/discussions)
- **Email**: damienprouet76@gmail.com

---

**Note:** This is an educational project focused on learning Rust and game development. The game is in active development and not yet playable.
