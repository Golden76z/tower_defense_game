use crate::game::enemies::enemy_base::Enemy;
use crate::game::projectiles::Projectile;
use crate::game::towers::manager::TowerType;
use glam::Vec2;

/// Represents a tower in the game.
///
/// This trait defines the core functionality that all towers must implement,
/// including updating their state, shooting at enemies, and providing their
/// position and range.
///
/// # Examples
///
/// ```rust,ignore
/// use glam::Vec2;
/// use crate::game::towers::tower_base::Tower;
/// use crate::game::enemies::enemy_base::Enemy;
/// use crate::game::projectiles::Projectile;
///
/// pub struct BasicTower {
///     position: Vec2,
///     range: f32,
///     cooldown: f32,
/// }
///
/// impl Tower for BasicTower {
///     fn update(&mut self, dt: f32, enemies: &[Box<dyn Enemy>]) -> Option<Projectile> {
///         self.cooldown -= dt;
///         if self.can_shoot() && !enemies.is_empty() {
///             self.cooldown = 1.0;
///             // Return a dummy projectile
///             Some(Projectile {})
///         } else {
///             None
///         }
///     }
///
///     fn can_shoot(&self) -> bool {
///         self.cooldown <= 0.0
///     }
///
///     fn get_position(&self) -> Vec2 {
///         self.position
///     }
///
///     fn get_range(&self) -> f32 {
///         self.range
///     }
/// }
/// ```
pub trait Tower {
    /// Updates the tower's state, potentially firing a projectile at enemies.
    ///
    /// * `dt`: The elapsed time since the last frame.
    /// * `enemies`: A slice of current enemies in the game.
    ///
    /// Returns an `Option<Projectile>` if the tower fires, otherwise `None`.
    fn update(
        &mut self,
        dt: f32,
        enemies: &[Box<dyn Enemy>],
        spatial_grid: Option<&crate::game::spatial_grid::SpatialGrid>,
        query_scratch: &mut Vec<usize>,
    ) -> Option<Projectile>;

    /// Checks if the tower is currently able to shoot.
    fn can_shoot(&self) -> bool;

    /// Gets the current position of the tower in the world.
    fn get_position(&self) -> Vec2;

    /// Gets the attack range of the tower.
    fn get_range(&self) -> f32;

    /// Gets the current rotation of the tower in radians.
    fn get_rotation(&self) -> f32;

    /// Gets the type of this tower.
    fn tower_type(&self) -> TowerType;

    /// Gets the current upgrade level of the tower (1-3).
    fn get_level(&self) -> u32;

    /// Upgrades the tower to the next level.
    fn upgrade(&mut self) -> Result<(), &'static str>;

    /// Gets the cost to upgrade to the next level. Returns None if max level is reached.
    fn get_upgrade_cost(&self) -> Option<i32>;

    /// Gets the current damage of the tower.
    fn get_damage(&self) -> f32;

    /// Gets the fire rate (shots per second) of the tower.
    fn get_fire_rate(&self) -> f32;
}
