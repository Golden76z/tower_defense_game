//! Waypoint path that enemies follow across the map.
//!
//! A [`Path`] is an ordered list of waypoints in **tile space** — the same
//! coordinates used to index the map, where `x` is the column and `y` is the
//! row (the world Z axis). Enemies walk from one waypoint to the next: ask for
//! the direction toward the current waypoint with [`Path::get_direction`], and
//! check whether it's been reached with [`Path::reached_waypoint`].
//!
//! Keeping waypoints in tile space (rather than rendered world units) means the
//! path lines up exactly with the `Path` tiles carved into the map, and is
//! independent of camera/zoom. Convert to world space at render time if needed.

use glam::Vec2;

/// How close (in tile units) an entity must get to a waypoint to count as
/// having reached it.
pub const DEFAULT_REACH_RADIUS: f32 = 0.1;

/// An ordered list of waypoints defining the route enemies follow.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path {
    waypoints: Vec<Vec2>,
}

impl Path {
    /// Creates a path from an ordered list of tile-space waypoints.
    pub fn new(waypoints: Vec<Vec2>) -> Self {
        Self { waypoints }
    }

    /// The waypoints, in order.
    pub fn waypoints(&self) -> &[Vec2] {
        &self.waypoints
    }

    /// Number of waypoints.
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// Whether the path has no waypoints.
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    /// The waypoint at `index`, if it exists.
    pub fn waypoint(&self, index: usize) -> Option<Vec2> {
        self.waypoints.get(index).copied()
    }

    /// The first waypoint (spawn point), if any.
    pub fn start(&self) -> Option<Vec2> {
        self.waypoints.first().copied()
    }

    /// Returns `true` once `waypoint_index` is past the end of the path, i.e.
    /// the entity has walked the whole route.
    pub fn is_finished(&self, waypoint_index: usize) -> bool {
        waypoint_index >= self.waypoints.len()
    }

    /// Unit direction from `position` toward the waypoint at `waypoint_index`.
    ///
    /// Returns [`Vec2::ZERO`] if the index is out of range or `position` is
    /// already exactly on the waypoint (so callers can multiply by speed safely
    /// without producing NaNs).
    pub fn get_direction(&self, position: Vec2, waypoint_index: usize) -> Vec2 {
        match self.waypoints.get(waypoint_index) {
            Some(&target) => (target - position).normalize_or_zero(),
            None => Vec2::ZERO,
        }
    }

    /// Whether `position` is within [`DEFAULT_REACH_RADIUS`] of the waypoint at
    /// `waypoint_index`. Out-of-range indices are never "reached".
    pub fn reached_waypoint(&self, position: Vec2, waypoint_index: usize) -> bool {
        self.reached_waypoint_within(position, waypoint_index, DEFAULT_REACH_RADIUS)
    }

    /// Like [`Path::reached_waypoint`] but with a caller-supplied radius.
    pub fn reached_waypoint_within(&self, position: Vec2, waypoint_index: usize, radius: f32) -> bool {
        match self.waypoints.get(waypoint_index) {
            Some(&target) => position.distance_squared(target) <= radius * radius,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example path from the issue:
    /// start (0,0) then waypoints (0,5), (5,5), (5,10), (10,10).
    fn example_path() -> Path {
        Path::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 5.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.0, 10.0),
            Vec2::new(10.0, 10.0),
        ])
    }

    fn assert_dir(actual: Vec2, expected: Vec2) {
        assert!(
            (actual - expected).length() < 1e-5,
            "direction {actual:?} != expected {expected:?}"
        );
    }

    #[test]
    fn basic_accessors() {
        let path = example_path();
        assert_eq!(path.len(), 5);
        assert!(!path.is_empty());
        assert_eq!(path.start(), Some(Vec2::new(0.0, 0.0)));
        assert_eq!(path.waypoint(2), Some(Vec2::new(5.0, 5.0)));
        assert_eq!(path.waypoint(99), None);
        assert!(Path::default().is_empty());
    }

    #[test]
    fn direction_points_at_next_waypoint() {
        let path = example_path();

        // From the start toward (0,5): straight along +Y.
        assert_dir(path.get_direction(Vec2::new(0.0, 0.0), 1), Vec2::new(0.0, 1.0));
        // From (0,5) toward (5,5): straight along +X.
        assert_dir(path.get_direction(Vec2::new(0.0, 5.0), 2), Vec2::new(1.0, 0.0));
        // From (5,5) toward (5,10): +Y again.
        assert_dir(path.get_direction(Vec2::new(5.0, 5.0), 3), Vec2::new(0.0, 1.0));
    }

    #[test]
    fn direction_is_normalised_for_diagonals() {
        let path = Path::new(vec![Vec2::ZERO, Vec2::new(3.0, 4.0)]);
        let dir = path.get_direction(Vec2::ZERO, 1);
        assert!((dir.length() - 1.0).abs() < 1e-5, "expected unit length, got {}", dir.length());
        assert_dir(dir, Vec2::new(0.6, 0.8)); // 3-4-5 triangle
    }

    #[test]
    fn direction_zero_when_on_target_or_out_of_range() {
        let path = example_path();
        assert_eq!(path.get_direction(Vec2::new(0.0, 5.0), 1), Vec2::ZERO); // already there
        assert_eq!(path.get_direction(Vec2::ZERO, 99), Vec2::ZERO); // no such waypoint
    }

    #[test]
    fn reached_waypoint_detects_arrival() {
        let path = example_path();

        // Exactly on the waypoint.
        assert!(path.reached_waypoint(Vec2::new(0.0, 5.0), 1));
        // Just within the reach radius.
        assert!(path.reached_waypoint(Vec2::new(0.05, 5.0), 1));
        // Clearly too far away.
        assert!(!path.reached_waypoint(Vec2::new(0.0, 4.0), 1));
        // Just outside the radius.
        assert!(!path.reached_waypoint(Vec2::new(0.0, 5.0 - DEFAULT_REACH_RADIUS - 0.01), 1));
    }

    #[test]
    fn reached_waypoint_respects_custom_radius() {
        let path = example_path();
        // 1.0 away from (0,5); within a radius of 1.5, not within 0.5.
        assert!(path.reached_waypoint_within(Vec2::new(0.0, 4.0), 1, 1.5));
        assert!(!path.reached_waypoint_within(Vec2::new(0.0, 4.0), 1, 0.5));
    }

    #[test]
    fn out_of_range_waypoint_never_reached() {
        let path = example_path();
        assert!(!path.reached_waypoint(Vec2::new(10.0, 10.0), 99));
    }

    #[test]
    fn is_finished_at_end_of_path() {
        let path = example_path();
        assert!(!path.is_finished(0));
        assert!(!path.is_finished(4)); // last valid waypoint
        assert!(path.is_finished(5)); // walked past the end
        assert!(path.is_finished(100));
    }

    #[test]
    fn full_walk_advances_through_every_waypoint() {
        // Simulate an entity stepping along the example path and assert it
        // visits each waypoint in order.
        let path = example_path();
        let mut pos = path.start().unwrap();
        let mut index = 1; // heading toward the second point
        let speed = 0.25;
        let mut visited = vec![pos];

        for _ in 0..100_000 {
            if path.is_finished(index) {
                break;
            }
            if path.reached_waypoint(pos, index) {
                pos = path.waypoint(index).unwrap();
                visited.push(pos);
                index += 1;
                continue;
            }
            pos += path.get_direction(pos, index) * speed;
        }

        assert!(path.is_finished(index), "entity should reach the end");
        assert_eq!(visited.last().copied(), Some(Vec2::new(10.0, 10.0)));
        assert_eq!(visited.len(), path.len());
    }
}
