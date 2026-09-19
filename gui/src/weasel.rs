//! The weasel -- *weasel* rhymes with *MDeezl* -- drawn from one list of
//! shapes, with no image file and no image decoding. The same list paints
//! the hero in the window and rasterises the window icon.
//!
//! egui fills only convex polygons, so the animal is built from discs and
//! "tubes": polylines whose points carry a radius, painted as round-jointed
//! thick segments. Coordinates are in a unit box, `[0,1] × [0,HEIGHT]`, with
//! y growing downwards as in egui.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke};

/// The unit box's height; its width is 1.
pub const HEIGHT: f32 = 0.6;

const FUR: Color32 = Color32::from_rgb(0x8b, 0x5a, 0x2b);
/// The far legs, a shade darker so they read as behind the body.
const FUR_SHADE: Color32 = Color32::from_rgb(0x70, 0x47, 0x22);
const CREAM: Color32 = Color32::from_rgb(0xf2, 0xe3, 0xc6);
const DARK: Color32 = Color32::from_rgb(0x2a, 0x1e, 0x14);
const INNER_EAR: Color32 = Color32::from_rgb(0xc9, 0x8f, 0x6a);

/// One piece of the weasel, in the unit box.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Disc {
        center: [f32; 2],
        radius: f32,
        color: Color32,
    },
    /// Points `[x, y, radius]`, joined by round-ended segments.
    Tube {
        points: Vec<[f32; 3]>,
        color: Color32,
    },
}

/// A tube along the cubic Bézier `p0 c1 c2 p3`, its radius easing from `r0`
/// to `r1`, sampled at `n` segments.
fn bezier_tube(
    [p0, c1, c2, p3]: [[f32; 2]; 4],
    (r0, r1): (f32, f32),
    n: usize,
    color: Color32,
) -> Shape {
    let points = (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            [
                a * p0[0] + b * c1[0] + c * c2[0] + d * p3[0],
                a * p0[1] + b * c1[1] + c * c2[1] + d * p3[1],
                r0 + (r1 - r0) * t,
            ]
        })
        .collect();
    Shape::Tube { points, color }
}

fn disc(x: f32, y: f32, radius: f32, color: Color32) -> Shape {
    Shape::Disc {
        center: [x, y],
        radius,
        color,
    }
}

/// The weasel, back to front: later shapes cover earlier ones.
pub fn shapes() -> Vec<Shape> {
    vec![
        // The tail: long, low, and black at the tip, like a stoat's.
        bezier_tube(
            [[0.30, 0.40], [0.20, 0.47], [0.12, 0.42], [0.08, 0.30]],
            (0.040, 0.024),
            12,
            FUR,
        ),
        bezier_tube(
            [[0.105, 0.365], [0.09, 0.34], [0.08, 0.32], [0.075, 0.28]],
            (0.028, 0.022),
            4,
            DARK,
        ),
        // The far legs, a shade behind the near ones.
        bezier_tube(
            [[0.34, 0.44], [0.33, 0.49], [0.31, 0.52], [0.30, 0.545]],
            (0.030, 0.024),
            4,
            FUR_SHADE,
        ),
        bezier_tube(
            [[0.70, 0.42], [0.71, 0.47], [0.72, 0.51], [0.73, 0.545]],
            (0.028, 0.022),
            4,
            FUR_SHADE,
        ),
        // The body: the arched, bounding back that makes a weasel a weasel.
        bezier_tube(
            [[0.30, 0.41], [0.36, 0.12], [0.60, 0.10], [0.74, 0.36]],
            (0.090, 0.066),
            24,
            FUR,
        ),
        // The near legs.
        bezier_tube(
            [[0.28, 0.44], [0.27, 0.49], [0.25, 0.52], [0.24, 0.55]],
            (0.030, 0.024),
            4,
            FUR,
        ),
        bezier_tube(
            [[0.76, 0.42], [0.77, 0.47], [0.78, 0.51], [0.79, 0.55]],
            (0.028, 0.022),
            4,
            FUR,
        ),
        // Neck and cream throat.
        bezier_tube(
            [[0.72, 0.37], [0.76, 0.34], [0.80, 0.33], [0.84, 0.32]],
            (0.064, 0.060),
            6,
            FUR,
        ),
        bezier_tube(
            [[0.745, 0.405], [0.78, 0.395], [0.82, 0.375], [0.86, 0.360]],
            (0.020, 0.024),
            6,
            CREAM,
        ),
        // Head, snout, ear, eye and nose; the cream chin tucks under the snout.
        disc(0.845, 0.305, 0.068, FUR),
        disc(0.880, 0.352, 0.028, CREAM),
        disc(0.915, 0.335, 0.040, FUR),
        disc(0.815, 0.245, 0.028, FUR),
        disc(0.815, 0.248, 0.015, INNER_EAR),
        disc(0.872, 0.292, 0.012, DARK),
        disc(0.876, 0.288, 0.004, Color32::WHITE),
        disc(0.952, 0.340, 0.013, DARK),
    ]
}

/// Map the unit box onto `rect`, centred, keeping its proportions.
fn to_screen(rect: Rect) -> impl Fn([f32; 2]) -> Pos2 {
    let scale = rect.width().min(rect.height() / HEIGHT);
    let origin = rect.center() - egui::vec2(0.5, HEIGHT / 2.0) * scale;
    move |[x, y]| origin + egui::vec2(x, y) * scale
}

/// Paint the weasel into `rect`: a filled circle per disc and per tube
/// point, and a line segment per tube segment.
pub fn paint(painter: &egui::Painter, rect: Rect) {
    let scale = rect.width().min(rect.height() / HEIGHT);
    let at = to_screen(rect);
    for shape in shapes() {
        match shape {
            Shape::Disc {
                center,
                radius,
                color,
            } => {
                painter.circle_filled(at(center), radius * scale, color);
            }
            Shape::Tube { points, color } => {
                for pair in points.windows(2) {
                    let [a, b] = [pair[0], pair[1]];
                    let width = (a[2] + b[2]) * scale;
                    painter.line_segment(
                        [at([a[0], a[1]]), at([b[0], b[1]])],
                        Stroke::new(width, color),
                    );
                }
                for p in &points {
                    painter.circle_filled(at([p[0], p[1]]), p[2] * scale, color);
                }
            }
        }
    }
}

/// Does `shape` cover the unit-box point `(x, y)`?
fn covers(shape: &Shape, x: f32, y: f32) -> bool {
    match shape {
        Shape::Disc { center, radius, .. } => {
            let (dx, dy) = (x - center[0], y - center[1]);
            dx * dx + dy * dy <= radius * radius
        }
        Shape::Tube { points, .. } => points.windows(2).any(|pair| {
            let (a, b) = (pair[0], pair[1]);
            let (abx, aby) = (b[0] - a[0], b[1] - a[1]);
            let len2 = abx * abx + aby * aby;
            let t = if len2 == 0.0 {
                0.0
            } else {
                (((x - a[0]) * abx + (y - a[1]) * aby) / len2).clamp(0.0, 1.0)
            };
            let (px, py) = (a[0] + abx * t - x, a[1] + aby * t - y);
            let r = a[2] + (b[2] - a[2]) * t;
            px * px + py * py <= r * r
        }),
    }
}

fn color_of(shape: &Shape) -> Color32 {
    match shape {
        Shape::Disc { color, .. } | Shape::Tube { color, .. } => *color,
    }
}

/// The window icon: a `size`×`size` RGBA raster of the same shapes, the unit
/// box centred on the square, 4×4 samples per pixel for smooth edges.
pub fn icon(size: u32) -> egui::IconData {
    const SAMPLES: u32 = 4;
    let shapes = shapes();
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let top = (1.0 - HEIGHT) / 2.0;
    for py in 0..size {
        for px in 0..size {
            let (mut sum, mut hits) = ([0u32; 3], 0u32);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let x = (px as f32 + (sx as f32 + 0.5) / SAMPLES as f32) / size as f32;
                    let y = (py as f32 + (sy as f32 + 0.5) / SAMPLES as f32) / size as f32 - top;
                    if let Some(shape) = shapes.iter().rev().find(|s| covers(s, x, y)) {
                        let c = color_of(shape);
                        sum[0] += c.r() as u32;
                        sum[1] += c.g() as u32;
                        sum[2] += c.b() as u32;
                        hits += 1;
                    }
                }
            }
            if hits == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                rgba.extend(sum.map(|s| (s / hits) as u8));
                rgba.push((hits * 255 / (SAMPLES * SAMPLES)) as u8);
            }
        }
    }
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}
