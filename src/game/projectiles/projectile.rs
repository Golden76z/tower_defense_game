use glam::Vec2;

pub struct Projectile {
    pub start_position: Vec2,
    pub position: Vec2,
    pub target_position: Vec2,
    pub speed: f32,
    pub damage: f32,
    pub spawn_height_offset: f32,
    pub alive: bool,
}

impl Projectile {
    pub fn new(
        position: Vec2,
        target_position: Vec2,
        speed: f32,
        damage: f32,
        spawn_height_offset: f32,
    ) -> Self {
        Self {
            start_position: position,
            position,
            target_position,
            speed,
            damage,
            spawn_height_offset,
            alive: true,
        }
    }

    /// Update projectile position. Returns true if the projectile has reached the target.
    pub fn update(&mut self, dt: f32) -> bool {
        if !self.alive {
            return true;
        }

        let direction = self.target_position - self.position;
        let distance = direction.length();

        let travel_distance = self.speed * dt;

        if travel_distance >= distance {
            // Reached target
            self.position = self.target_position;
            self.alive = false;
            return true;
        }

        self.position += direction.normalize() * travel_distance;
        false
    }
}

/// Resolves a single-target projectile hit: the index of the closest candidate
/// to `impact` lying within `radius`, or `None` if none are in range.
/// `candidates` are `(index, position)` pairs of the enemies eligible to be hit
/// (e.g. the alive ones). Used so a single-target tower damages exactly one
/// enemy instead of everything clustered near the impact point.
pub fn closest_target_within(
    impact: Vec2,
    radius: f32,
    candidates: &[(usize, Vec2)],
) -> Option<usize> {
    candidates
        .iter()
        .map(|&(idx, pos)| (idx, pos.distance(impact)))
        .filter(|&(_, dist)| dist <= radius)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(idx, _)| idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_target_within_picks_nearest_in_range() {
        let impact = Vec2::ZERO;
        let candidates = [
            (0usize, Vec2::new(0.4, 0.0)), // in range
            (1usize, Vec2::new(0.2, 0.0)), // nearest, in range
            (2usize, Vec2::new(0.9, 0.0)), // out of range
        ];
        assert_eq!(closest_target_within(impact, 0.5, &candidates), Some(1));
        // Nothing within a tiny radius.
        assert_eq!(closest_target_within(impact, 0.1, &candidates), None);
        // No candidates at all.
        assert_eq!(closest_target_within(impact, 0.5, &[]), None);
    }

    #[test]
    fn test_projectile_movement() {
        let mut proj = Projectile::new(Vec2::ZERO, Vec2::new(10.0, 0.0), 5.0, 10.0, 0.8);

        let reached = proj.update(1.0);
        assert!(!reached);
        assert_eq!(proj.position.x, 5.0);
        assert!(proj.alive);

        let reached = proj.update(1.0);
        assert!(reached);
        assert_eq!(proj.position.x, 10.0);
        assert!(!proj.alive);
    }
}
