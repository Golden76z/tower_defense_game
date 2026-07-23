pub struct SpatialGrid {
    cols: usize,
    rows: usize,
    cell_size: f32,
    cells: Vec<Vec<usize>>,
}

impl SpatialGrid {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        let cols = ((width as f32 / cell_size).ceil() as usize).max(1);
        let rows = ((height as f32 / cell_size).ceil() as usize).max(1);
        let num_cells = cols * rows;
        let mut cells = Vec::with_capacity(num_cells);
        for _ in 0..num_cells {
            cells.push(Vec::with_capacity(16));
        }
        Self {
            cols,
            rows,
            cell_size,
            cells,
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    pub fn insert(&mut self, enemy_index: usize, position: glam::Vec2) {
        if position.x < 0.0 || position.y < 0.0 {
            return;
        }
        let cx = (position.x / self.cell_size) as usize;
        let cy = (position.y / self.cell_size) as usize;
        if cx < self.cols && cy < self.rows {
            let cell_idx = cy * self.cols + cx;
            self.cells[cell_idx].push(enemy_index);
        }
    }

    pub fn query(&self, center: glam::Vec2, radius: f32, result: &mut Vec<usize>) {
        result.clear();
        let min_x = center.x - radius;
        let max_x = center.x + radius;
        let min_y = center.y - radius;
        let max_y = center.y + radius;

        let min_cx =
            ((min_x / self.cell_size).floor() as i32).clamp(0, self.cols as i32 - 1) as usize;
        let max_cx =
            ((max_x / self.cell_size).floor() as i32).clamp(0, self.cols as i32 - 1) as usize;
        let min_cy =
            ((min_y / self.cell_size).floor() as i32).clamp(0, self.rows as i32 - 1) as usize;
        let max_cy =
            ((max_y / self.cell_size).floor() as i32).clamp(0, self.rows as i32 - 1) as usize;

        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                let cell_idx = cy * self.cols + cx;
                result.extend_from_slice(&self.cells[cell_idx]);
            }
        }
    }
}
