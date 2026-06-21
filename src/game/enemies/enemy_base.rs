use glam::Vec2;
use crate::game::map::path::Path;

/// Represents an enemy in the game.
///
/// This trait defines the core functionality that all enemies must implement,
/// including movement along a path, taking damage, and tracking their current
/// health and position.
///
/// # Requirements
///
/// Any struct implementing this trait should manage its own internal state
/// for health, position, and any other specific attributes it needs. The
/// `update` method should handle moving the enemy along the provided path based
/// on the elapsed time `dt`.
///
/// # Examples
///
/// ```rust,ignore
/// use glam::Vec2;
/// use crate::game::map::path::Path;
/// use crate::game::enemies::enemy_base::Enemy;
///
/// pub struct BasicEnemy {
///     position: Vec2,
///     health: f32,
/// }
///
/// impl Enemy for BasicEnemy {
///     fn update(&mut self, dt: f32, path: &Path) {
///         // Logic to follow the path
///     }
///
///     fn take_damage(&mut self, amount: f32) -> bool {
///         self.health -= amount;
///         self.is_alive()
///     }
///
///     fn get_position(&self) -> Vec2 {
///         self.position
///     }
///
///     fn get_health(&self) -> f32 {
///         self.health
///     }
///
///     fn is_alive(&self) -> bool {
///         self.health > 0.0
///     }
/// }
/// ```
pub trait Enemy {
    /// Updates the enemy's state, such as moving along a path.
    ///
    /// * `dt`: The elapsed time since the last frame.
    /// * `path`: The path the enemy should follow.
    fn update(&mut self, dt: f32, path: &Path);

    /// Applies damage to the enemy.
    ///
    /// * `amount`: The amount of damage to take.
    /// Returns `true` if the enemy is still alive after taking damage, `false` otherwise.
    fn take_damage(&mut self, amount: f32) -> bool;

    /// Gets the current position of the enemy in the world.
    fn get_position(&self) -> Vec2;

    /// Gets the current health of the enemy.
    fn get_health(&self) -> f32;

    /// Checks if the enemy is currently alive.
    fn is_alive(&self) -> bool;

    /// Gets the gold reward for defeating this enemy.
    fn get_reward(&self) -> u32;
}
