use crate::layout::Point;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl Bounds {
    pub(crate) fn from_points(points: impl IntoIterator<Item = Point>) -> Self {
        points
            .into_iter()
            .fold(Self::empty(), |bounds, point| bounds.include(point))
    }

    pub(crate) fn empty() -> Self {
        Self {
            min_x: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            min_y: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    fn include(mut self, point: Point) -> Self {
        self.min_x = self.min_x.min(point.x);
        self.max_x = self.max_x.max(point.x);
        self.min_y = self.min_y.min(point.y);
        self.max_y = self.max_y.max(point.y);
        self
    }

    pub(crate) fn padded(self) -> Self {
        Self {
            min_x: padded_bound(self.min_x, self.max_x, true),
            max_x: padded_bound(self.min_x, self.max_x, false),
            min_y: padded_bound(self.min_y, self.max_y, true),
            max_y: padded_bound(self.min_y, self.max_y, false),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Viewport {
    pub bounds: Bounds,
    preserve_aspect: bool,
    margin: bool,
}

impl Viewport {
    pub(crate) fn plot(bounds: Bounds) -> Self {
        Self {
            bounds,
            preserve_aspect: false,
            margin: false,
        }
    }

    pub(crate) fn with_aspect(bounds: Bounds) -> Self {
        Self {
            bounds,
            preserve_aspect: true,
            margin: true,
        }
    }

    pub(crate) fn project(self, point: Point, width: usize, height: usize) -> Pixel {
        let (left, right, top, bottom) = if self.margin {
            let margin_x = width.min(6) as f64 / 2.0;
            let margin_y = height.min(6) as f64 / 2.0;
            (
                margin_x,
                width.saturating_sub(1) as f64 - margin_x,
                margin_y,
                height.saturating_sub(1) as f64 - margin_y,
            )
        } else {
            (
                0.0,
                width.saturating_sub(1) as f64,
                0.0,
                height.saturating_sub(1) as f64,
            )
        };
        let x_range = range(self.bounds.min_x, self.bounds.max_x);
        let y_range = range(self.bounds.min_y, self.bounds.max_y);
        if !self.preserve_aspect {
            return Pixel {
                x: left + (point.x - self.bounds.min_x) / x_range * (right - left).max(1.0),
                y: bottom - (point.y - self.bounds.min_y) / y_range * (bottom - top).max(1.0),
            };
        }
        let x_pixels = ((right - left).max(1.0)) * 2.0;
        let y_pixels = ((bottom - top).max(1.0)) * 4.0;
        let scale = (x_pixels / x_range).min(y_pixels / y_range);
        let plot_width = x_range * scale;
        let plot_height = y_range * scale;
        let origin_x = (left * 2.0 + (x_pixels - plot_width) / 2.0) / 2.0;
        let origin_y = (top * 4.0 + (y_pixels - plot_height) / 2.0) / 4.0;
        Pixel {
            x: origin_x + (point.x - self.bounds.min_x) * scale / 2.0,
            y: origin_y + (y_range - (point.y - self.bounds.min_y)) * scale / 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Pixel {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct PlotNode {
    pub position: Point,
    pub label: Option<String>,
    pub owner: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PlotEdge {
    pub from: usize,
    pub to: usize,
    pub owner: Option<usize>,
}

pub(crate) type PlotPixels<'a> = (
    Vec<Vec<u8>>,
    Vec<Vec<Option<usize>>>,
    Vec<(Pixel, Option<&'a str>)>,
);

#[derive(Clone, Debug, Default)]
pub(crate) struct Plot {
    pub nodes: Vec<PlotNode>,
    pub edges: Vec<PlotEdge>,
}
impl Plot {
    pub(crate) fn bounds(&self) -> Bounds {
        Bounds::from_points(self.nodes.iter().map(|node| node.position))
    }

    pub(crate) fn pixels(&self, viewport: Viewport, width: usize, height: usize) -> PlotPixels<'_> {
        let mut dots = vec![vec![0_u8; width]; height];
        let mut owners = vec![vec![None; width]; height];
        for edge in &self.edges {
            if let (Some(from), Some(to)) = (self.nodes.get(edge.from), self.nodes.get(edge.to)) {
                draw_line(
                    &mut dots,
                    &mut owners,
                    edge_endpoint(from, to, viewport, width, height),
                    edge_endpoint(to, from, viewport, width, height),
                    edge.owner,
                );
            }
        }
        let nodes = self
            .nodes
            .iter()
            .map(|node| {
                let pixel = viewport.project(node.position, width, height);
                put_node(&mut dots, &mut owners, pixel, node.owner);
                (pixel, node.label.as_deref())
            })
            .collect();
        (dots, owners, nodes)
    }
}

fn edge_endpoint(
    node: &PlotNode,
    other: &PlotNode,
    viewport: Viewport,
    width: usize,
    height: usize,
) -> Pixel {
    let center = viewport.project(node.position, width, height);
    let Some(label) = node.label.as_deref() else {
        return center;
    };
    let target = viewport.project(other.position, width, height);
    let dx = target.x - center.x;
    let dy = target.y - center.y;
    let half_width = (label.chars().count() + 2) as f64 / 2.0;
    let half_height = 0.5;
    let scale =
        (half_width / dx.abs().max(f64::EPSILON)).min(half_height / dy.abs().max(f64::EPSILON));
    Pixel {
        x: center.x + dx * scale.min(1.0),
        y: center.y + dy * scale.min(1.0),
    }
}

fn range(min: f64, max: f64) -> f64 {
    (max - min).max(f64::EPSILON)
}

fn padded_bound(min: f64, max: f64, lower: bool) -> f64 {
    if min < max {
        let padding = (max - min) * 0.12;
        return if lower { min - padding } else { max + padding };
    }
    let padding = min.abs().max(1.0) * 0.05;
    if lower { min - padding } else { max + padding }
}

fn draw_line(
    canvas: &mut [Vec<u8>],
    owners: &mut [Vec<Option<usize>>],
    from: Pixel,
    to: Pixel,
    owner: Option<usize>,
) {
    let Some(((x0, y0), (x1, y1))) = clip_line(
        from.x * 2.0,
        from.y * 4.0,
        to.x * 2.0,
        to.y * 4.0,
        canvas[0].len() as f64 * 2.0 - 1.0,
        canvas.len() as f64 * 4.0 - 1.0,
    ) else {
        return;
    };
    let (mut x, mut y) = (x0.round() as isize, y0.round() as isize);
    let (x1, y1) = (x1.round() as isize, y1.round() as isize);
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
    let mut error = dx + dy;
    loop {
        put_dot(
            canvas,
            owners,
            Pixel {
                x: x as f64 / 2.0,
                y: y as f64 / 4.0,
            },
            owner,
        );
        if x == x1 && y == y1 {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x += sx;
        }
        if twice <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn put_node(
    canvas: &mut [Vec<u8>],
    owners: &mut [Vec<Option<usize>>],
    pixel: Pixel,
    owner: Option<usize>,
) {
    let x = (pixel.x * 2.0).round() as isize;
    let y = (pixel.y * 4.0).round() as isize;
    if x < 0 || y < 0 {
        return;
    }
    let cell_x = x as usize / 2;
    let cell_y = y as usize / 4;
    if let Some(cell) = canvas.get_mut(cell_y).and_then(|row| row.get_mut(cell_x)) {
        *cell |= 0b0011_0110;
        owners[cell_y][cell_x] = owner.or(owners[cell_y][cell_x]);
    }
}

fn put_dot(
    canvas: &mut [Vec<u8>],
    owners: &mut [Vec<Option<usize>>],
    pixel: Pixel,
    owner: Option<usize>,
) {
    let x = (pixel.x * 2.0).round() as isize;
    let y = (pixel.y * 4.0).round() as isize;
    put_micro_dot(canvas, owners, x, y, owner);
}

fn put_micro_dot(
    canvas: &mut [Vec<u8>],
    owners: &mut [Vec<Option<usize>>],
    x: isize,
    y: isize,
    owner: Option<usize>,
) {
    if x < 0 || y < 0 {
        return;
    }
    const DOTS: [[u8; 2]; 4] = [[1, 8], [2, 16], [4, 32], [64, 128]];
    let cell_x = x as usize / 2;
    let cell_y = y as usize / 4;
    if let Some(cell) = canvas.get_mut(cell_y).and_then(|row| row.get_mut(cell_x)) {
        *cell |= DOTS[y as usize % 4][x as usize % 2];
        owners[cell_y][cell_x] = owner.or(owners[cell_y][cell_x]);
    }
}

fn clip_line(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    max_x: f64,
    max_y: f64,
) -> Option<((f64, f64), (f64, f64))> {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let (mut start, mut end): (f64, f64) = (0.0, 1.0);
    for (p, q) in [(-dx, x0), (dx, max_x - x0), (-dy, y0), (dy, max_y - y0)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let ratio = q / p;
            if p < 0.0 {
                if ratio > end {
                    return None;
                }
                start = start.max(ratio);
            } else {
                if ratio < start {
                    return None;
                }
                end = end.min(ratio);
            }
        }
    }
    let clipped_x0 = x0 + start * dx;
    let clipped_y0 = y0 + start * dy;
    let clipped_x1 = x0 + end * dx;
    let clipped_y1 = y0 + end * dy;
    Some((
        (clipped_x0.clamp(0.0, max_x), clipped_y0.clamp(0.0, max_y)),
        (clipped_x1.clamp(0.0, max_x), clipped_y1.clamp(0.0, max_y)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padded_bounds_add_margin_to_non_degenerate_ranges() {
        let bounds = Bounds {
            min_x: 0.0,
            max_x: 10.0,
            min_y: -5.0,
            max_y: 5.0,
        }
        .padded();
        assert_eq!(bounds.min_x, -1.2);
        assert_eq!(bounds.max_x, 11.2);
        assert_eq!(bounds.min_y, -6.2);
        assert_eq!(bounds.max_y, 6.2);
    }

    #[test]
    fn line_clipping_keeps_intersections_inside_canvas() {
        let clipped = clip_line(-100.0, 10.0, 100.0, 10.0, 79.0, 39.0).unwrap();
        assert_eq!(clipped.0, (0.0, 10.0));
        assert_eq!(clipped.1, (79.0, 10.0));
    }

    #[test]
    fn line_clipping_does_not_extend_an_in_bounds_segment() {
        let clipped = clip_line(10.0, 8.0, 30.0, 24.0, 79.0, 39.0).unwrap();
        assert_eq!(clipped.0, (10.0, 8.0));
        assert_eq!(clipped.1, (30.0, 24.0));
    }

    #[test]
    fn plot_rasterizes_nodes_and_edges() {
        let plot = Plot {
            nodes: vec![
                PlotNode {
                    position: Point { x: 0.0, y: 0.0 },
                    label: Some("a".into()),
                    owner: Some(2),
                },
                PlotNode {
                    position: Point { x: 1.0, y: 1.0 },
                    label: None,
                    owner: Some(2),
                },
            ],
            edges: vec![PlotEdge {
                from: 0,
                to: 1,
                owner: Some(2),
            }],
        };
        let bounds = plot.bounds().padded();
        let (dots, owners, nodes) = plot.pixels(Viewport::plot(bounds), 20, 10);
        assert!(dots.iter().flatten().any(|dots| *dots == 0b0011_0110));
        assert!(owners.iter().flatten().any(|owner| *owner == Some(2)));
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].1, Some("a"));
    }
}
