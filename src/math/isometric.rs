use glam::{Vec2, Vec3};

/// 2:1 ratio for isometric
pub const TILE_WIDTH: f32 = 64.0;
pub const TILE_HEIGHT: f32 = 32.0;

/// Converts a 3D world coordinate to a 2D screen coordinate in an isometric projection.
/// Assuming:
/// - X axis points down-right on the screen.
/// - Z axis points down-left on the screen.
/// - Y axis is the elevation (pointing UP in the world, moving UP on the screen).
#[inline]
pub fn world_to_screen(world: Vec3) -> Vec2 {
    let half_width = TILE_WIDTH / 2.0;
    let half_height = TILE_HEIGHT / 2.0;

    let screen_x = (world.x - world.z) * half_width;
    // We subtract world.y * TILE_HEIGHT so that a positive Y moves UP on the screen (assuming screen Y points down).
    // If the rendering system expects screen Y to point UP, this might need to be positive.
    let screen_y = (world.x + world.z) * half_height - (world.y * TILE_HEIGHT);

    Vec2::new(screen_x, screen_y)
}

/// Converts a 2D screen coordinate back to a 3D world coordinate, given a specific elevation (Y).
#[inline]
pub fn screen_to_world(screen: Vec2, elevation_y: f32) -> Vec3 {
    let half_width = TILE_WIDTH / 2.0;
    let half_height = TILE_HEIGHT / 2.0;

    // Adjust the screen Y to the ground plane by adding the elevation offset back
    let adjusted_screen_y = screen.y + (elevation_y * TILE_HEIGHT);

    let world_x = (screen.x / half_width + adjusted_screen_y / half_height) / 2.0;
    let world_z = (adjusted_screen_y / half_height - screen.x / half_width) / 2.0;

    Vec3::new(world_x, elevation_y, world_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-4;

    fn assert_vec3_approx_eq(a: Vec3, b: Vec3) {
        assert!(
            (a.x - b.x).abs() < EPSILON &&
            (a.y - b.y).abs() < EPSILON &&
            (a.z - b.z).abs() < EPSILON,
            "Vectors are not approximately equal: {:?} vs {:?}", a, b
        );
    }

    #[test]
    fn test_origin() {
        let world = Vec3::ZERO;
        let screen = world_to_screen(world);
        assert_eq!(screen, Vec2::ZERO);

        let round_trip = screen_to_world(screen, 0.0);
        assert_vec3_approx_eq(world, round_trip);
    }

    #[test]
    fn test_world_to_screen_multiple_positions() {
        // Step in +X goes down-right
        assert_eq!(world_to_screen(Vec3::new(1.0, 0.0, 0.0)), Vec2::new(32.0, 16.0));
        
        // Step in +Z goes down-left
        assert_eq!(world_to_screen(Vec3::new(0.0, 0.0, 1.0)), Vec2::new(-32.0, 16.0));
        
        // Step in +Y goes purely UP (negative Y in screen coords)
        assert_eq!(world_to_screen(Vec3::new(0.0, 1.0, 0.0)), Vec2::new(0.0, -32.0));
        
        // Complex position
        assert_eq!(world_to_screen(Vec3::new(2.0, 1.5, 3.0)), Vec2::new(-32.0, 32.0));
    }

    #[test]
    fn test_round_trip() {
        let test_cases = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(5.0, 2.5, -3.0),
            Vec3::new(-100.5, 50.0, 20.25),
        ];

        for world in test_cases {
            let screen = world_to_screen(world);
            let round_trip = screen_to_world(screen, world.y);
            assert_vec3_approx_eq(world, round_trip);
        }
    }
}
