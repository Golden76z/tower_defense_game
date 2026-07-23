//! Stateless rendering helpers extracted from `window.rs`: ray/AABB tile
//! picking, terrain mesh generation, and CPU-side text-texture builders.

use crate::engine::window::{TERRAIN_BASE_LEVEL, TILE_WORLD_SIZE};

/// Ray vs axis-aligned box intersection (slab method).
///
/// Returns the distance along `dir` to the nearest entry point (clamped to 0
/// when the origin is already inside), or `None` if the ray misses the box.
/// `dir` is expected to be normalised.
pub fn ray_aabb(
    origin: glam::Vec3,
    dir: glam::Vec3,
    min: glam::Vec3,
    max: glam::Vec3,
) -> Option<f32> {
    let inv = glam::Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
    let t1 = (min - origin) * inv;
    let t2 = (max - origin) * inv;

    let t_enter = t1.min(t2).max_element();
    let t_exit = t1.max(t2).min_element();

    if t_exit >= t_enter.max(0.0) {
        Some(t_enter.max(0.0))
    } else {
        None
    }
}

/// Builds a face-culled terrain mesh from the map into `batcher`, then uploads
/// it to the GPU. Only visible faces are generated:
///
/// * the top face of every tile, and
/// * side faces only where a tile is taller than its neighbour (the exposed
///   "cliff"); faces touching an equal-or-taller neighbour are skipped.
///
/// Map-border tiles emit a short skirt down to `TERRAIN_BASE_LEVEL` so the underside of
/// the island isn't visible from the isometric camera angle. This replaces the
/// old approach of stacking full cubes from a fixed `-4` foundation, which
/// generated huge amounts of hidden geometry every frame.
pub fn build_terrain_mesh(
    map: &crate::game::map::Map,
    batcher: &mut crate::renderer::batch::SpriteBatcher,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    use crate::game::map::tile::TileType;
    use crate::renderer::batch::Face;

    let cube_size = glam::Vec3::splat(TILE_WORLD_SIZE);

    batcher.clear();

    let w = map.width;
    let h = map.height;

    for x in 0..w {
        for z in 0..h {
            let tile = match map.get_tile(x, z) {
                Some(t) => t,
                None => continue,
            };

            let pos_x = (x as f32 - w as f32 / 2.0) * TILE_WORLD_SIZE;
            let pos_z = (z as f32 - h as f32 / 2.0) * TILE_WORLD_SIZE;
            let top_h = tile.grid_y;

            // Texture IDs: 0 = grass side, 1 = grass top, 2 = water, 3 = sand, 4 = rock.
            let (top_tex, side_tex) = match tile.tile_type {
                TileType::Grass => (1usize, 0usize),
                TileType::Water => (2, 2),
                TileType::Sand => (3, 3),
                TileType::Rock => (4, 4),
                TileType::Path => (3, 0), // Use sand for path for now.
            };

            // Top face — always visible from above.
            batcher.add_face(
                glam::Vec3::new(pos_x, top_h as f32 * cube_size.y, pos_z),
                cube_size,
                Face::Top,
                top_tex,
            );

            // Exposed side skirts: one face per level between the neighbour's
            // top and ours. Levels at or below the neighbour are hidden.
            let neighbours = [
                (x + 1, z, Face::East),
                (x - 1, z, Face::West),
                (x, z + 1, Face::South),
                (x, z - 1, Face::North),
            ];

            for (nx, nz, face) in neighbours {
                let neighbour_h =
                    map.get_tile(nx, nz).map(|t| t.grid_y).unwrap_or(TERRAIN_BASE_LEVEL);

                let mut level = neighbour_h + 1;
                while level <= top_h {
                    batcher.add_face(
                        glam::Vec3::new(pos_x, level as f32 * cube_size.y, pos_z),
                        cube_size,
                        face,
                        side_tex,
                    );
                    level += 1;
                }
            }
        }
    }

    batcher.finalize(device, queue);
}

pub fn create_gold_texture(text: &str) -> image::RgbaImage {
    let width = 32;
    let height = 16;
    let mut img = image::RgbaImage::new(width, height);

    // Tiny 3x5 font: 0-9 and '+'
    let get_char_bitmap = |c: char| -> &'static [u8; 5] {
        match c {
            '+' => &[2, 2, 7, 2, 2],
            '0' => &[7, 5, 5, 5, 7],
            '1' => &[2, 6, 2, 2, 7],
            '2' => &[7, 1, 7, 4, 7],
            '3' => &[7, 1, 7, 1, 7],
            '4' => &[5, 5, 7, 1, 1],
            '5' => &[7, 4, 7, 1, 7],
            '6' => &[7, 4, 7, 5, 7],
            '7' => &[7, 1, 2, 2, 2],
            '8' => &[7, 5, 7, 5, 7],
            '9' => &[7, 5, 7, 1, 7],
            _ => &[0, 0, 0, 0, 0],
        }
    };

    // Calculate total width to center the text
    let char_width = 3;
    let spacing = 1;
    let total_width = text.len() as i32 * char_width + (text.len() as i32 - 1) * spacing;
    let start_x = (width as i32 - total_width) / 2;
    let start_y = (height as i32 - 5) / 2;

    for (char_idx, c) in text.chars().enumerate() {
        let bitmap = get_char_bitmap(c);
        let cx = start_x + char_idx as i32 * (char_width + spacing);

        for (row, &val) in bitmap.iter().enumerate() {
            let cy = start_y + row as i32;
            for col in 0..3 {
                // Check if bit (2 - col) is set
                let bit = 1 << (2 - col);
                if (val & bit) != 0 {
                    let px = cx + col;
                    if px >= 0 && px < width as i32 && cy >= 0 && cy < height as i32 {
                        img.put_pixel(px as u32, cy as u32, image::Rgba([255, 220, 0, 255]));
                    }
                }
            }
        }
    }

    // Outline pass: for any yellow pixel, check its 8 neighbors. If they are empty, make them black.
    let mut yellow_pixels = std::collections::HashSet::new();
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] > 0 && p[0] == 255 && p[1] == 220 {
                yellow_pixels.insert((x as i32, y as i32));
            }
        }
    }

    for &(x, y) in &yellow_pixels {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0
                    && nx < width as i32
                    && ny >= 0
                    && ny < height as i32
                    && !yellow_pixels.contains(&(nx, ny))
                {
                    img.put_pixel(nx as u32, ny as u32, image::Rgba([0, 0, 0, 255]));
                }
            }
        }
    }

    img
}

pub fn create_multiline_text_texture(
    line1: &str,
    line2: &str,
    width: u32,
    height: u32,
    color1: [u8; 4],
    color2: [u8; 4],
) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(width, height);

    // Tiny 3x5 font
    let get_char_bitmap = |c: char| -> &'static [u8; 5] {
        match c {
            '0' => &[7, 5, 5, 5, 7],
            '1' => &[2, 2, 2, 2, 2],
            '2' => &[7, 1, 7, 4, 7],
            '3' => &[7, 1, 7, 1, 7],
            '4' => &[5, 5, 7, 1, 1],
            '5' => &[7, 4, 7, 1, 7],
            '6' => &[7, 4, 7, 5, 7],
            '7' => &[7, 1, 1, 1, 1],
            '8' => &[7, 5, 7, 5, 7],
            '9' => &[7, 5, 7, 1, 7],
            'A' | 'a' => &[2, 5, 7, 5, 5],
            'B' | 'b' => &[6, 5, 6, 5, 6],
            'C' | 'c' => &[7, 4, 4, 4, 7],
            'D' | 'd' => &[6, 5, 5, 5, 6],
            'E' | 'e' => &[7, 4, 6, 4, 7],
            'F' | 'f' => &[7, 4, 6, 4, 4],
            'G' | 'g' => &[7, 4, 5, 5, 7],
            'H' | 'h' => &[5, 5, 7, 5, 5],
            'I' | 'i' => &[7, 2, 2, 2, 7],
            'J' | 'j' => &[1, 1, 1, 5, 2],
            'K' | 'k' => &[5, 5, 6, 5, 5],
            'L' | 'l' => &[4, 4, 4, 4, 7],
            'M' | 'm' => &[5, 7, 5, 5, 5],
            'N' | 'n' => &[5, 7, 7, 5, 5],
            'O' | 'o' => &[7, 5, 5, 5, 7],
            'P' | 'p' => &[7, 5, 7, 4, 4],
            'Q' | 'q' => &[7, 5, 5, 7, 1],
            'R' | 'r' => &[6, 5, 6, 5, 5],
            'S' | 's' => &[7, 4, 7, 1, 7],
            'T' | 't' => &[7, 2, 2, 2, 2],
            'U' | 'u' => &[5, 5, 5, 5, 7],
            'V' | 'v' => &[5, 5, 5, 5, 2],
            'W' | 'w' => &[5, 5, 5, 7, 5],
            'X' | 'x' => &[5, 5, 2, 5, 5],
            'Y' | 'y' => &[5, 5, 2, 2, 2],
            'Z' | 'z' => &[7, 1, 2, 4, 7],
            '.' => &[0, 0, 0, 0, 2],
            ':' => &[0, 2, 0, 2, 0],
            '!' => &[2, 2, 2, 0, 2],
            '+' => &[2, 2, 7, 2, 2],
            '-' => &[0, 0, 7, 0, 0],
            ' ' => &[0, 0, 0, 0, 0],
            _ => &[0, 0, 0, 0, 0],
        }
    };

    // Draw line 1 (vertical center of top half, e.g. height / 4 = 12 for height = 48)
    if !line1.is_empty() {
        let text_width = line1.len() as i32 * 4 - 1;
        let start_x = (width as i32 - text_width) / 2;
        let start_y = (height as i32 / 4) - 2;

        for (char_idx, c) in line1.chars().enumerate() {
            let bitmap = get_char_bitmap(c);
            let cx = start_x + char_idx as i32 * 4;

            for (row, &row_val) in bitmap.iter().enumerate() {
                let cy = start_y + row as i32;
                for col in 0..3 {
                    let bit = (row_val >> (2 - col)) & 1;
                    if bit == 1 {
                        let px = cx + col;
                        if px >= 0 && px < width as i32 && cy >= 0 && cy < height as i32 {
                            img.put_pixel(px as u32, cy as u32, image::Rgba(color1));
                        }
                    }
                }
            }
        }
    }

    // Draw line 2 (vertical center of bottom half, e.g. 3 * height / 4 = 36 for height = 48)
    if !line2.is_empty() {
        let text_width = line2.len() as i32 * 4 - 1;
        let start_x = (width as i32 - text_width) / 2;
        let start_y = (3 * height as i32 / 4) - 2;

        for (char_idx, c) in line2.chars().enumerate() {
            let bitmap = get_char_bitmap(c);
            let cx = start_x + char_idx as i32 * 4;

            for (row, &row_val) in bitmap.iter().enumerate() {
                let cy = start_y + row as i32;
                for col in 0..3 {
                    let bit = (row_val >> (2 - col)) & 1;
                    if bit == 1 {
                        let px = cx + col;
                        if px >= 0 && px < width as i32 && cy >= 0 && cy < height as i32 {
                            img.put_pixel(px as u32, cy as u32, image::Rgba(color2));
                        }
                    }
                }
            }
        }
    }

    // Outline pass: for any pixel of our text colors, check its 8 neighbors. If they are empty, make them black.
    let mut text_pixels = std::collections::HashSet::new();
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            let is_colored1 =
                p[3] > 0 && p[0] == color1[0] && p[1] == color1[1] && p[2] == color1[2];
            let is_colored2 =
                p[3] > 0 && p[0] == color2[0] && p[1] == color2[1] && p[2] == color2[2];
            if is_colored1 || is_colored2 {
                text_pixels.insert((x as i32, y as i32));
            }
        }
    }

    for &(x, y) in &text_pixels {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0
                    && nx < width as i32
                    && ny >= 0
                    && ny < height as i32
                    && !text_pixels.contains(&(nx, ny))
                {
                    img.put_pixel(nx as u32, ny as u32, image::Rgba([0, 0, 0, 255]));
                }
            }
        }
    }

    img
}

pub fn draw_minimap_preview(ui: &mut egui::Ui, grid: &[[u32; 10]; 10]) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
        for row in grid {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                for &cell in row {
                    let color = match cell {
                        1 => egui::Color32::from_rgb(218, 165, 32), // Path (gold)
                        2 => egui::Color32::from_rgb(120, 120, 120), // Rock (grey)
                        _ => egui::Color32::from_rgb(34, 139, 34),  // Grass (forest green)
                    };
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 1.0, color);
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_aabb_hits_box_in_front() {
        // Ray starting at origin pointing +Z into a unit box on the +Z axis.
        let origin = glam::Vec3::new(0.0, 0.0, 0.0);
        let dir = glam::Vec3::new(0.0, 0.0, 1.0);
        let min = glam::Vec3::new(-0.5, -0.5, 2.0);
        let max = glam::Vec3::new(0.5, 0.5, 3.0);

        let t = ray_aabb(origin, dir, min, max).expect("ray should hit the box");
        assert!(
            (t - 2.0).abs() < 1e-5,
            "entry distance should be 2.0, got {t}"
        );
    }

    #[test]
    fn ray_aabb_misses_box_to_the_side() {
        let origin = glam::Vec3::new(0.0, 0.0, 0.0);
        let dir = glam::Vec3::new(0.0, 0.0, 1.0);
        // Box is offset in X so the +Z ray never enters it.
        let min = glam::Vec3::new(5.0, -0.5, 2.0);
        let max = glam::Vec3::new(6.0, 0.5, 3.0);

        assert!(ray_aabb(origin, dir, min, max).is_none());
    }

    #[test]
    fn ray_aabb_behind_is_missed() {
        // Box is entirely behind the ray origin (negative Z), ray points +Z.
        let origin = glam::Vec3::new(0.0, 0.0, 0.0);
        let dir = glam::Vec3::new(0.0, 0.0, 1.0);
        let min = glam::Vec3::new(-0.5, -0.5, -3.0);
        let max = glam::Vec3::new(0.5, 0.5, -2.0);

        assert!(ray_aabb(origin, dir, min, max).is_none());
    }
}
