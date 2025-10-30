// src/lib.rs
// This file exposes the game logic as a library for testing
// The binary (main.rs) depends on this library

pub mod engine;
pub mod game;
pub mod math;
pub mod renderer;

// Re-export commonly used types for easier testing
// pub use game::{
//     enemies::Enemy,
//     towers::Tower,
//     map::Map,
// };

// pub use math::isometric::{screen_to_world, world_to_screen};

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_library_compiles() {
        // Sanity check that the library structure is valid
        assert!(true);
    }
}
