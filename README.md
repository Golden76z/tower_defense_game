# Tower Defense Game

A 2D isometric tower defense game built from scratch in Rust with a custom game engine.

[![CI](https://github.com/yourusername/tower_defense_game/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/tower_defense_game/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 🎮 About

Defend against waves of enemies by strategically placing towers, building walls, and upgrading your arsenal. Features a progression system with card-based upgrades, multiple biomes, boss battles, and an AI versus mode.

**Status:** 🚧 In Development (Phase 0 - Foundation)

## ✨ Features (Planned)

- **Core Tower Defense**: Classic TD gameplay with multiple tower types
- **Building System**: Place towers and walls with rotation support
- **Advanced Combat**: AOE damage, projectile reflection, multiple targeting modes
- **Progression System**: XP levels with card-based upgrade choices (Vampire Survivors style)
- **Campaign Mode**: 50+ stages across 5 unique biomes with story elements
- **Boss Battles**: Epic encounters at the end of each biome
- **Versus AI**: 1v1 Age of War style battles against AI opponent
- **Multiple Game Modes**: Endless mode, daily challenges, achievements

## 🛠️ Built With

- **Language**: Rust 2021 Edition
- **Graphics**: wgpu (modern, cross-platform rendering)
- **Windowing**: winit
- **Math**: glam
- **UI**: egui (planned)

## 🚀 Quick Start

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

### Development Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Use the test script
./scripts/run_tests.sh
```

## 📋 Project Structure

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

<!-- ## 📖 Documentation -->

<!-- - [Roadmap](ROADMAP.md) - Complete development plan -->
<!-- - [Testing Guide](TESTING_GUIDE.md) - How to write and run tests -->
<!-- - [Setup Checklist](SETUP_CHECKLIST.md) - Initial setup guide -->
<!-- - [GitHub Issues](GITHUB_ISSUES.md) - Detailed task breakdown -->

<!-- ## 🎯 Current Milestone -->
<!---->
<!-- **Phase 0: Foundation** - Setting up project structure and learning Rust basics -->
<!---->
<!-- See [ROADMAP.md](ROADMAP.md) for detailed phase breakdown. -->
<!---->
<!-- ## 🤝 Contributing -->
<!---->
<!-- This is a learning project, but contributions are welcome! Please: -->
<!---->
<!-- 1. Check the [issues](https://github.com/yourusername/tower_defense_game/issues) page -->
<!-- 2. Comment on an issue you'd like to work on -->
<!-- 3. Fork the repository and create a feature branch -->
<!-- 4. Make your changes with tests -->
<!-- 5. Submit a pull request -->
<!---->
<!-- See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines. -->

## 📝 License

This project is dual-licensed under:

- MIT License ([LICENSE](LICENSE))

## 🙏 Acknowledgments

- Inspired by classic TD games and Vampire Survivors
- Built with guidance from the Rust gamedev community
- wgpu tutorial by [@sotrh](https://sotrh.github.io/learn-wgpu/)

## 📬 Contact

- **Issues**: [GitHub Issues](https://github.com/yourusername/tower_defense_game/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/tower_defense_game/discussions)

---

**Note:** This is an educational project focused on learning Rust and game development. The game is in active development and not yet playable.
