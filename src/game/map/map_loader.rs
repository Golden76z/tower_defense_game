//! Loading map data from JSON configuration files.
//!
//! # JSON format
//!
//! ```json
//! {
//!   "width": 10,
//!   "height": 10,
//!   "tiles": [
//!     [0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
//!     ...
//!   ]
//! }
//! ```
//!
//! `tiles` is a row-major grid of tile-type ids, addressed as `tiles[row][col]`
//! (i.e. `tiles[y][x]`). There must be exactly `height` rows, each with exactly
//! `width` ids.
//!
//! ## Tile ids
//!
//! | id | tile  | walkable | stack height |
//! |----|-------|----------|--------------|
//! | 0  | Grass | yes      | 0            |
//! | 1  | Path  | yes      | 0            |
//! | 2  | Rock  | no       | 1            |
//!
//! Even though the source data is a flat 2D grid, the game is 3D: each tile is
//! given a vertical stack height (`grid_y`) so it integrates with the
//! isometric terrain renderer. Grass and path sit at sea level (0) to form a
//! flat playfield, while rock is raised one step to read as an obstacle.

use std::fmt;
use std::path::Path;

use glam::Vec2;
use serde::Deserialize;

use super::map::Map;
use super::tile::{Tile, TileType};
// NOTE: `super::path::Path` is referenced fully-qualified below to avoid a name
// clash with `std::path::Path` (imported above for file loading).

/// Raw map data as stored in a JSON file.
///
/// `tiles` is row-major: `tiles[y][x]` is the tile-type id at column `x`,
/// row `y`.
#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Vec<u32>>,
    /// Optional enemy route as `[x, y]` tile-space waypoints. Absent in older
    /// maps, in which case the path is empty.
    #[serde(default)]
    pub path: Vec<[f32; 2]>,
}

/// Errors that can occur while loading a map from JSON.
#[derive(Debug)]
pub enum MapLoadError {
    /// The file could not be read from disk.
    Io(std::io::Error),
    /// The contents were not valid JSON or did not match the expected schema.
    Parse(serde_json::Error),
    /// `width` or `height` was not strictly positive.
    InvalidDimensions { width: i32, height: i32 },
    /// The number of rows did not match the declared `height`.
    RowCountMismatch { expected: i32, found: usize },
    /// A row's length did not match the declared `width`.
    RowLengthMismatch {
        row: usize,
        expected: i32,
        found: usize,
    },
    /// A tile id in the grid has no known mapping.
    UnknownTile { row: usize, col: usize, id: u32 },
}

impl fmt::Display for MapLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapLoadError::Io(e) => write!(f, "failed to read map file: {e}"),
            MapLoadError::Parse(e) => write!(f, "failed to parse map JSON: {e}"),
            MapLoadError::InvalidDimensions { width, height } => write!(
                f,
                "invalid map dimensions: width={width}, height={height} (both must be > 0)"
            ),
            MapLoadError::RowCountMismatch { expected, found } => write!(
                f,
                "row count mismatch: expected {expected} rows, found {found}"
            ),
            MapLoadError::RowLengthMismatch {
                row,
                expected,
                found,
            } => write!(
                f,
                "row {row} length mismatch: expected {expected} columns, found {found}"
            ),
            MapLoadError::UnknownTile { row, col, id } => {
                write!(f, "unknown tile id {id} at row {row}, column {col}")
            }
        }
    }
}

impl std::error::Error for MapLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MapLoadError::Io(e) => Some(e),
            MapLoadError::Parse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MapLoadError {
    fn from(e: std::io::Error) -> Self {
        MapLoadError::Io(e)
    }
}

impl From<serde_json::Error> for MapLoadError {
    fn from(e: serde_json::Error) -> Self {
        MapLoadError::Parse(e)
    }
}

/// Maps a JSON tile id to its `(TileType, stack height, walkable)`.
///
/// Returns `None` for unrecognised ids so the caller can surface a precise
/// [`MapLoadError::UnknownTile`].
fn tile_from_id(id: u32) -> Option<(TileType, i32, bool)> {
    match id {
        0 => Some((TileType::Grass, 0, true)),
        1 => Some((TileType::Path, 0, true)),
        2 => Some((TileType::Rock, 1, false)),
        _ => None,
    }
}

impl MapData {
    /// Validates the raw data and converts it into a [`Map`].
    ///
    /// Mapping follows the renderer's convention: column `x` -> `grid_x`,
    /// stack height -> `grid_y`, row `y` -> `grid_z`.
    pub fn into_map(self) -> Result<Map, MapLoadError> {
        if self.width <= 0 || self.height <= 0 {
            return Err(MapLoadError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        }

        if self.tiles.len() != self.height as usize {
            return Err(MapLoadError::RowCountMismatch {
                expected: self.height,
                found: self.tiles.len(),
            });
        }

        let mut map = Map::new(self.width, self.height);

        for (y, row) in self.tiles.iter().enumerate() {
            if row.len() != self.width as usize {
                return Err(MapLoadError::RowLengthMismatch {
                    row: y,
                    expected: self.width,
                    found: row.len(),
                });
            }

            for (x, &id) in row.iter().enumerate() {
                let (tile_type, level, walkable) =
                    tile_from_id(id).ok_or(MapLoadError::UnknownTile { row: y, col: x, id })?;

                // x -> grid_x, level -> grid_y (vertical), y -> grid_z.
                let tile = Tile::new(tile_type, x as i32, level, y as i32, walkable);

                // Indices are guaranteed in-bounds by the checks above, but
                // propagate just in case the invariant ever changes.
                map.set_tile(x as i32, y as i32, tile).map_err(|_| {
                    MapLoadError::InvalidDimensions {
                        width: self.width,
                        height: self.height,
                    }
                })?;
            }
        }

        map.path =
            super::path::Path::new(self.path.iter().map(|p| Vec2::new(p[0], p[1])).collect());

        Ok(map)
    }
}

/// Loads a map from a JSON string.
pub fn load_map_from_str(json: &str) -> Result<Map, MapLoadError> {
    let data: MapData = serde_json::from_str(json)?;
    data.into_map()
}

/// Loads a map from a JSON file at `path`.
pub fn load_map_from_file<P: AsRef<Path>>(path: P) -> Result<Map, MapLoadError> {
    let contents = std::fs::read_to_string(path)?;
    load_map_from_str(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_valid_map_and_maps_tile_types() {
        let json = r#"{ "width": 3, "height": 2, "tiles": [[0, 1, 2], [2, 1, 0]] }"#;
        let map = load_map_from_str(json).expect("valid map should load");

        assert_eq!(map.width, 3);
        assert_eq!(map.height, 2);
        assert_eq!(map.tiles.len(), 6);

        assert_eq!(map.get_tile(0, 0).unwrap().tile_type, TileType::Grass);
        assert_eq!(map.get_tile(1, 0).unwrap().tile_type, TileType::Path);
        assert_eq!(map.get_tile(2, 0).unwrap().tile_type, TileType::Rock);
    }

    #[test]
    fn applies_3d_coordinates_and_walkability() {
        let json = r#"{ "width": 3, "height": 1, "tiles": [[0, 1, 2]] }"#;
        let map = load_map_from_str(json).unwrap();

        let path = map.get_tile(1, 0).unwrap();
        assert_eq!(path.grid_x, 1, "column maps to grid_x");
        assert_eq!(path.grid_z, 0, "row maps to grid_z");
        assert_eq!(path.grid_y, 0, "path sits at sea level");
        assert!(path.walkable);

        let rock = map.get_tile(2, 0).unwrap();
        assert_eq!(rock.grid_y, 1, "rock is raised one step");
        assert!(!rock.walkable, "rock is not walkable");
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(matches!(
            load_map_from_str("{ not valid json"),
            Err(MapLoadError::Parse(_))
        ));
    }

    #[test]
    fn rejects_non_positive_dimensions() {
        let json = r#"{ "width": 0, "height": 2, "tiles": [] }"#;
        assert!(matches!(
            load_map_from_str(json),
            Err(MapLoadError::InvalidDimensions { .. })
        ));
    }

    #[test]
    fn rejects_row_count_mismatch() {
        let json = r#"{ "width": 2, "height": 3, "tiles": [[0, 0], [0, 0]] }"#;
        assert!(matches!(
            load_map_from_str(json),
            Err(MapLoadError::RowCountMismatch { expected: 3, found: 2 })
        ));
    }

    #[test]
    fn rejects_row_length_mismatch() {
        let json = r#"{ "width": 3, "height": 1, "tiles": [[0, 0]] }"#;
        assert!(matches!(
            load_map_from_str(json),
            Err(MapLoadError::RowLengthMismatch { row: 0, expected: 3, found: 2 })
        ));
    }

    #[test]
    fn rejects_unknown_tile_id() {
        let json = r#"{ "width": 2, "height": 1, "tiles": [[0, 9]] }"#;
        assert!(matches!(
            load_map_from_str(json),
            Err(MapLoadError::UnknownTile { row: 0, col: 1, id: 9 })
        ));
    }
}
