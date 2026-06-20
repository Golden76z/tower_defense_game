#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Grass,
    Path,
    Rock,
    Water,
    Sand,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Grass
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub tile_type: TileType,
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_z: i32,
    pub walkable: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            tile_type: TileType::Grass,
            grid_x: 0,
            grid_y: 0,
            grid_z: 0,
            walkable: true,
        }
    }
}

impl Tile {
    pub fn new(tile_type: TileType, grid_x: i32, grid_y: i32, grid_z: i32, walkable: bool) -> Self {
        Self {
            tile_type,
            grid_x,
            grid_y,
            grid_z,
            walkable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_default() {
        let tile = Tile::default();
        assert_eq!(tile.tile_type, TileType::Grass);
        assert_eq!(tile.grid_x, 0);
        assert_eq!(tile.grid_y, 0);
        assert_eq!(tile.grid_z, 0);
        assert_eq!(tile.walkable, true);
    }

    #[test]
    fn test_tile_new() {
        let tile = Tile::new(TileType::Rock, 1, 2, 3, false);
        assert_eq!(tile.tile_type, TileType::Rock);
        assert_eq!(tile.grid_x, 1);
        assert_eq!(tile.grid_y, 2);
        assert_eq!(tile.grid_z, 3);
        assert_eq!(tile.walkable, false);
    }

    #[test]
    fn test_tile_clone() {
        let tile = Tile::new(TileType::Path, 5, 0, -5, true);
        let cloned = tile.clone();
        assert_eq!(tile, cloned);
    }
}
