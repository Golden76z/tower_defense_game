use super::tile::{Tile, TileType};
use noise::{Fbm, NoiseFn, SuperSimplex};

pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::default(); (width * height) as usize],
        }
    }

    pub fn get_tile(&self, x: i32, z: i32) -> Option<&Tile> {
        if x >= 0 && x < self.width && z >= 0 && z < self.height {
            Some(&self.tiles[(z * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, x: i32, z: i32) -> Option<&mut Tile> {
        if x >= 0 && x < self.width && z >= 0 && z < self.height {
            Some(&mut self.tiles[(z * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn generate_island(width: i32, height: i32, seed: u32) -> Self {
        let mut map = Self::new(width, height);
        let fbm = Fbm::<SuperSimplex>::new(seed);
        
        let center_x = width as f32 / 2.0;
        let center_z = height as f32 / 2.0;
        let max_distance = (center_x.powi(2) + center_z.powi(2)).sqrt();

        for x in 0..width {
            for z in 0..height {
                // Get base noise value
                // Scale coordinates so the noise isn't too high frequency
                let nx = x as f64 * 0.1;
                let nz = z as f64 * 0.1;
                let mut noise_val = fbm.get([nx, nz]) as f32; // ranges from roughly -1 to 1

                // Calculate distance from center for the island mask
                let dx = x as f32 - center_x;
                let dz = z as f32 - center_z;
                let distance = (dx * dx + dz * dz).sqrt();
                
                // Mask the noise based on distance to ensure edges are water
                let normalized_dist = distance / max_distance;
                // Subtract more as we get closer to the edge
                let falloff = normalized_dist.powi(2) * 2.5; 
                noise_val -= falloff;

                // Map noise value to discrete heights
                // Water is at height_steps = -1 and 0, Sand at 1, Grass at 2..4, Rock above 4
                let height_steps = (noise_val * 4.0).round() as i32;

                let tile_type = if height_steps <= -1 {
                    TileType::Water
                } else if height_steps == 0 {
                    TileType::Sand
                } else if height_steps >= 4 {
                    TileType::Rock
                } else {
                    TileType::Grass
                };

                let walkable = match tile_type {
                    TileType::Water => false,
                    TileType::Rock => false,
                    _ => true,
                };

                let tile = Tile::new(tile_type, x, height_steps, z, walkable);
                
                if let Some(t) = map.get_tile_mut(x, z) {
                    *t = tile;
                }
            }
        }

        map
    }
}
