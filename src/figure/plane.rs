/// A reusable Cartesian plane. Coordinates are independent of terminal size.
#[derive(Clone, Copy)]
pub struct Plane {
    pub center_x: f64,
    pub center_y: f64,
    pub scale: f64,
    pub show_axes: bool,
    pub show_grid: bool,
}

impl Plane {
    pub fn new(show_axes: bool, show_grid: bool) -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            scale: 1.25,
            show_axes,
            show_grid,
        }
    }

    pub fn project(self, x: f64, y: f64, width: usize, height: usize) -> Option<(usize, usize)> {
        let (x, y) = self.project_unclipped(x, y, width, height);
        (x >= 0 && y >= 0 && x < width as isize && y < height as isize)
            .then_some((x as usize, y as usize))
    }

    /// Converts a plane coordinate without discarding off-screen geometry.
    /// Renderers use this for line clipping at their raster boundary.
    pub fn project_unclipped(self, x: f64, y: f64, width: usize, height: usize) -> (isize, isize) {
        let x = ((x - self.center_x) / self.scale + 1.0) * width as f64 / 2.0;
        let y = (1.0 - (y - self.center_y) / self.scale) * height as f64 / 2.0;
        (x.round() as isize, y.round() as isize)
    }

    pub fn pan(&mut self, x: i8, y: i8) {
        self.center_x += f64::from(x) * self.scale * 0.15;
        self.center_y -= f64::from(y) * self.scale * 0.15;
    }

    pub fn zoom(&mut self, factor: f64) {
        self.scale = (self.scale * factor).clamp(0.1, 20.0);
    }
}

pub fn background(plane: Plane, width: usize, height: usize) -> Vec<Vec<char>> {
    let mut cells = vec![vec![' '; width]; height];
    if plane.show_grid {
        draw_grid(&mut cells, plane);
    }
    if plane.show_axes {
        draw_axes(&mut cells, plane);
    }
    cells
}

fn draw_grid(cells: &mut [Vec<char>], plane: Plane) {
    for coordinate in -10..=10 {
        let value = f64::from(coordinate);
        if let Some((x, _)) = plane.project(value, 0.0, cells[0].len(), cells.len()) {
            for row in cells.iter_mut() {
                row[x] = '·';
            }
        }
        if let Some((_, y)) = plane.project(0.0, value, cells[0].len(), cells.len()) {
            for cell in &mut cells[y] {
                *cell = '·';
            }
        }
    }
}

fn draw_axes(cells: &mut [Vec<char>], plane: Plane) {
    if let Some((x, _)) = plane.project(0.0, 0.0, cells[0].len(), cells.len()) {
        for row in cells.iter_mut() {
            row[x] = '│';
        }
    }
    if let Some((_, y)) = plane.project(0.0, 0.0, cells[0].len(), cells.len()) {
        for cell in &mut cells[y] {
            *cell = '─';
        }
    }
}
