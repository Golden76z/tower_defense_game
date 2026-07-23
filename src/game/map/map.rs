use super::path::Path;
use super::tile::{Tile, TileType};
use noise::{Fbm, MultiFractal, NoiseFn, SuperSimplex};

/// Radial island mask: `1.0` at the centre, easing smoothly down to `0.0`
/// at the rim. `dist` is the normalised distance from centre (0..1).
///
/// Using a smoothstep curve here (instead of a hard `dist^2` subtraction)
/// is what gives the coastline a soft, natural taper rather than an abrupt
/// wall of water.
fn island_mask(dist: f32) -> f32 {
    let t = (1.0 - dist).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    /// Waypoint route enemies follow across this map (tile-space coordinates).
    /// Empty when the map has no defined path.
    pub path: Path,
    /// Support for multiple paths (Hard map / bonus feature)
    pub paths: Vec<Path>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::default(); (width * height) as usize],
            path: Path::default(),
            paths: Vec::new(),
        }
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some(&self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some(&mut self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) -> Result<(), &'static str> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile;
            Ok(())
        } else {
            Err("Out of bounds")
        }
    }

    pub fn generate_island(width: i32, height: i32, seed: u32) -> Self {
        let mut map = Self::new(width, height);

        // Low-frequency fractal noise produces smooth, rolling terrain rather
        // than the single-tile spikes a high-frequency sample would create.
        // A handful of octaves layer in gentle detail on top of the broad shape.
        let fbm = Fbm::<SuperSimplex>::new(seed)
            .set_octaves(4)
            .set_frequency(0.06)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        let center_x = (width - 1) as f32 / 2.0;
        let center_y = (height - 1) as f32 / 2.0;
        let max_distance = (center_x.powi(2) + center_y.powi(2)).sqrt().max(1.0);

        for x in 0..width {
            for y in 0..height {
                // Sample noise (roughly -1..1) and remap to 0..1.
                let raw = fbm.get([x as f64, y as f64]) as f32;
                let noise01 = (raw * 0.5 + 0.5).clamp(0.0, 1.0);

                // Distance from the centre, normalised to 0..1.
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = ((dx * dx + dy * dy).sqrt() / max_distance).clamp(0.0, 1.0);
                let mask = island_mask(dist);

                // The radial dome decides the overall shape (so the island is
                // always ringed by water); the noise just textures the surface
                // so coastlines and peaks look organic instead of perfectly
                // circular.
                let elevation = (mask * (0.72 + 0.32 * noise01)).clamp(0.0, 1.0);

                // Classify elevation into terrain bands. The thin sand band
                // forms a beach wherever land meets the sea.
                let tile_type = if elevation < 0.30 {
                    TileType::Water
                } else if elevation < 0.38 {
                    TileType::Sand
                } else if elevation < 0.78 {
                    TileType::Grass
                } else {
                    TileType::Rock
                };

                // Convert elevation to a discrete stack height. Water sits at a
                // flat sea level (0); land rises smoothly above it. Because the
                // underlying field is smooth, neighbouring tiles differ by at
                // most a step, giving gentle terraced slopes.
                let height_steps = match tile_type {
                    TileType::Water => 0,
                    _ => 1 + ((elevation - 0.30) * 7.0).round() as i32,
                };

                let walkable =
                    matches!(tile_type, TileType::Grass | TileType::Sand | TileType::Path);

                // Here we map x -> grid_x, height_steps -> grid_y, y -> grid_z
                let tile = Tile::new(tile_type, x, height_steps, y, walkable);

                let _ = map.set_tile(x, y, tile);
            }
        }

        // Carve a straight road across the middle of the island and record it
        // as the enemy waypoint route. Without a path, no enemies spawn and the
        // level would instantly register as won.
        let road_y = height / 2;
        if width > 0 && road_y >= 0 && road_y < height {
            for x in 0..width {
                // Keep the terrain height but lift the road to at least land
                // level so enemies never appear to walk below the sea.
                let road_height = map.get_tile(x, road_y).map(|t| t.grid_y.max(1)).unwrap_or(1);
                let _ = map.set_tile(
                    x,
                    road_y,
                    Tile::new(TileType::Path, x, road_height, road_y, true),
                );
            }
            map.path = Path::new(vec![
                glam::Vec2::new(0.0, road_y as f32),
                glam::Vec2::new((width - 1) as f32, road_y as f32),
            ]);
        }

        map.paths = vec![map.path.clone()];
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_creation() {
        let map = Map::new(20, 20);
        assert_eq!(map.width, 20);
        assert_eq!(map.height, 20);
        assert_eq!(map.tiles.len(), 400);
    }

    #[test]
    fn test_get_tile_valid() {
        let map = Map::new(20, 20);
        let tile = map.get_tile(10, 10);
        assert!(tile.is_some());
    }

    #[test]
    fn test_get_tile_invalid() {
        let map = Map::new(20, 20);
        // Out of bounds cases
        assert!(map.get_tile(-1, 5).is_none());
        assert!(map.get_tile(5, -1).is_none());
        assert!(map.get_tile(20, 5).is_none());
        assert!(map.get_tile(5, 20).is_none());
    }

    #[test]
    fn test_set_tile() {
        let mut map = Map::new(20, 20);
        let custom_tile = Tile::new(TileType::Rock, 5, 2, 5, false);

        let result = map.set_tile(5, 5, custom_tile.clone());
        assert!(result.is_ok());

        let tile = map.get_tile(5, 5).unwrap();
        assert_eq!(tile.tile_type, TileType::Rock);
        assert!(!tile.walkable);
    }

    #[test]
    fn test_set_tile_invalid() {
        let mut map = Map::new(20, 20);
        let custom_tile = Tile::new(TileType::Rock, 5, 2, 5, false);

        assert!(map.set_tile(-1, 5, custom_tile.clone()).is_err());
        assert!(map.set_tile(20, 5, custom_tile.clone()).is_err());
    }

    #[test]
    fn generated_island_has_a_playable_path() {
        // Regression: generate_island used to leave `path` empty, so the
        // volcanic / procedural levels spawned no enemies and instantly "won".
        let map = Map::generate_island(20, 20, 1337);
        assert!(!map.path.is_empty(), "island must define a waypoint path");
        assert!(map.path.len() >= 2, "path needs a start and an end");
        assert!(map.path.start().is_some());
        // The spawner reads map.paths; it must expose the same non-empty route.
        assert!(!map.paths.is_empty());
        assert!(map.paths.iter().all(|p| !p.is_empty()));
        // Waypoints stay inside the map bounds.
        for wp in map.path.waypoints() {
            assert!(
                wp.x >= 0.0 && wp.x < map.width as f32,
                "x in bounds: {wp:?}"
            );
            assert!(
                wp.y >= 0.0 && wp.y < map.height as f32,
                "y in bounds: {wp:?}"
            );
        }
    }
}
