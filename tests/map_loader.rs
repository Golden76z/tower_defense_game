//! Integration tests for loading maps from JSON configuration files.

use std::path::PathBuf;

use tower_defense_lib::game::map::tile::TileType;
use tower_defense_lib::game::map::{load_map_from_file, load_map_from_str, MapLoadError};

/// Absolute path to the bundled test map, resolved relative to the crate root
/// so the test works regardless of the current working directory.
fn test_map_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/maps/test_map.json")
}

#[test]
fn loads_bundled_test_map() {
    let map = load_map_from_file(test_map_path()).expect("test_map.json should load");

    assert_eq!(map.width, 10);
    assert_eq!(map.height, 10);
    assert_eq!(map.tiles.len(), 100);

    // The bundled map's first row is all grass.
    for x in 0..10 {
        assert_eq!(map.get_tile(x, 0).unwrap().tile_type, TileType::Grass);
    }

    // It contains at least one path tile and at least one (non-walkable) rock.
    let has_path = map
        .tiles
        .iter()
        .any(|t| t.tile_type == TileType::Path && t.walkable);
    let has_rock = map
        .tiles
        .iter()
        .any(|t| t.tile_type == TileType::Rock && !t.walkable);
    assert!(has_path, "test map should contain a path");
    assert!(has_rock, "test map should contain a rock obstacle");
}

#[test]
fn loads_path_waypoints_that_lie_on_path_tiles() {
    let map = load_map_from_file(test_map_path()).expect("test_map.json should load");

    assert_eq!(map.path.len(), 7, "test map defines 7 waypoints");
    assert_eq!(map.path.start(), Some(glam::Vec2::new(0.0, 1.0)));
    assert_eq!(map.path.waypoint(6), Some(glam::Vec2::new(8.0, 9.0)));

    // Every waypoint must sit on a walkable Path tile so enemies can follow it.
    for wp in map.path.waypoints() {
        let tile = map
            .get_tile(wp.x as i32, wp.y as i32)
            .expect("waypoint should be in bounds");
        assert_eq!(tile.tile_type, TileType::Path, "waypoint {wp:?} off the path");
        assert!(tile.walkable, "waypoint {wp:?} not walkable");
    }
}

#[test]
fn maps_ids_to_3d_tiles() {
    let json = r#"{ "width": 3, "height": 2, "tiles": [[0, 1, 2], [2, 1, 0]] }"#;
    let map = load_map_from_str(json).expect("valid map should load");

    // 0 = Grass, 1 = Path, 2 = Rock.
    assert_eq!(map.get_tile(0, 0).unwrap().tile_type, TileType::Grass);
    assert_eq!(map.get_tile(1, 0).unwrap().tile_type, TileType::Path);
    assert_eq!(map.get_tile(2, 0).unwrap().tile_type, TileType::Rock);

    // Column -> grid_x, row -> grid_z; rock is raised in the 3D world.
    let rock = map.get_tile(0, 1).unwrap();
    assert_eq!(rock.grid_x, 0);
    assert_eq!(rock.grid_z, 1);
    assert_eq!(rock.grid_y, 1);
    assert!(!rock.walkable);
}

#[test]
fn invalid_json_returns_parse_error() {
    let result = load_map_from_str("{ this is not valid json");
    assert!(matches!(result, Err(MapLoadError::Parse(_))));
}

#[test]
fn missing_file_returns_io_error() {
    let result = load_map_from_file("definitely/does/not/exist.json");
    assert!(matches!(result, Err(MapLoadError::Io(_))));
}

#[test]
fn dimension_mismatch_returns_error() {
    // Declares 3 rows but provides 2.
    let json = r#"{ "width": 2, "height": 3, "tiles": [[0, 0], [0, 0]] }"#;
    assert!(matches!(
        load_map_from_str(json),
        Err(MapLoadError::RowCountMismatch { .. })
    ));
}

#[test]
fn unknown_tile_id_returns_error() {
    let json = r#"{ "width": 2, "height": 1, "tiles": [[0, 7]] }"#;
    assert!(matches!(
        load_map_from_str(json),
        Err(MapLoadError::UnknownTile { id: 7, .. })
    ));
}
