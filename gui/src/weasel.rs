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

/// The points of a tube along the cubic Bézier `p0 c1 c2 p3`, its radius
/// easing from `r0` to `r1`, sampled at `n` segments.
fn bezier_points([p0, c1, c2, p3]: [[f32; 2]; 4], (r0, r1): (f32, f32), n: usize) -> Vec<[f32; 3]> {
    (0..=n)
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
        .collect()
}

fn bezier_tube(curve: [[f32; 2]; 4], radii: (f32, f32), n: usize, color: Color32) -> Shape {
    Shape::Tube {
        points: bezier_points(curve, radii, n),
        color,
    }
}

/// One Bézier of a chain: its four control points, its radii, its samples.
type Segment = ([[f32; 2]; 4], (f32, f32), usize);

/// The points along several Béziers in a row, joined without repeating the
/// shared points: how the back gets its two humps in a single body.
fn chain_points(segments: &[Segment]) -> Vec<[f32; 3]> {
    let mut points: Vec<[f32; 3]> = Vec::new();
    for (curve, radii, n) in segments {
        let part = bezier_points(*curve, *radii, *n);
        let skip = usize::from(!points.is_empty());
        points.extend(part.into_iter().skip(skip));
    }
    points
}

/// A thinner tube whose *lower edge* is the lower edge of `points`, with its
/// radius a constant fraction of theirs. The pale belly then follows the
/// body's outline exactly, and widens at the same rate the body does. Both
/// ends ease in over `taper` points, so it fades into the fur instead of
/// stopping with a blunt end.
fn underside(points: &[[f32; 3]], fraction: f32, taper: usize) -> Vec<[f32; 3]> {
    (0..points.len())
        .map(|i| {
            let from_end = points.len() - 1 - i;
            let ramp = i.min(from_end).min(taper) + 1;
            let fraction = fraction * ramp as f32 / (taper + 1) as f32;
            // Straight down, so the pale edge lands on the body's lowest
            // point at this station: the bottom of the chest and the neck,
            // not the front of them, however steeply the body is falling.
            let [x, y, r] = points[i];
            let thin = r * fraction;
            [x, y + (r - thin), thin]
        })
        .collect()
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
    // The tail, and its dark tip taken from the tail's own last points, so
    // the tip is the same outline: it widens exactly as the tail does.
    let tail = bezier_points(
        [
            [0.175, 0.405],
            [0.115, 0.475],
            [0.060, 0.445],
            [0.018, 0.34],
        ],
        (0.040, 0.018),
        14,
    );
    let tip = tail[tail.len() - 5..].to_vec();

    // The back: two humps, so it reads as the M of MDeezl. The body sits to
    // the left of centre to leave the neck room to reach out.
    let spine = chain_points(&[
        (
            [[0.17, 0.400], [0.18, 0.20], [0.24, 0.10], [0.32, 0.10]],
            (0.078, 0.072),
            10,
        ),
        (
            [[0.32, 0.10], [0.39, 0.10], [0.405, 0.28], [0.46, 0.28]],
            (0.072, 0.068),
            10,
        ),
        (
            [[0.46, 0.28], [0.515, 0.28], [0.53, 0.10], [0.60, 0.10]],
            (0.068, 0.066),
            10,
        ),
        // Down the shoulder, leaving it at 45 degrees.
        (
            [[0.60, 0.10], [0.700, 0.10], [0.715, 0.325], [0.755, 0.365]],
            (0.066, 0.058),
            12,
        ),
        // The neck: long, reaching well out in front of the shoulder, and
        // clearly narrower than the body.
        (
            [
                [0.755, 0.365],
                [0.790, 0.395],
                [0.840, 0.365],
                [0.878, 0.338],
            ],
            (0.052, 0.044),
            10,
        ),
    ]);
    // The pale throat: the underside of the neck, from the chest to the chin,
    // taken from the neck's own points, so it traces that outline and widens
    // exactly as the neck does. Both ends fade into the fur. Traced further
    // back it becomes a second colour along the whole inside of the M, which
    // reads as a two-tone letter rather than as an animal.
    let throat = underside(&spine[spine.len() - 9..], 0.30, 3);

    vec![
        Shape::Tube {
            points: tail,
            color: FUR,
        },
        Shape::Tube {
            points: tip,
            color: DARK,
        },
        // The far hind leg, a shade behind the near one.
        bezier_tube(
            [[0.222, 0.435], [0.217, 0.48], [0.212, 0.52], [0.207, 0.555]],
            (0.030, 0.024),
            4,
            FUR_SHADE,
        ),
        Shape::Tube {
            points: spine,
            color: FUR,
        },
        Shape::Tube {
            points: throat,
            color: CREAM,
        },
        // The near hind leg, and the near fore leg under the chest, both in
        // front of the pale underside.
        bezier_tube(
            [[0.165, 0.435], [0.160, 0.48], [0.155, 0.52], [0.150, 0.555]],
            (0.030, 0.024),
            4,
            FUR,
        ),
        bezier_tube(
            [
                [0.757, 0.425],
                [0.763, 0.47],
                [0.769, 0.515],
                [0.775, 0.555],
            ],
            (0.028, 0.022),
            4,
            FUR,
        ),
        // The far fore leg, on the shoulder and in front of the pale chest,
        // so it is joined to the body rather than floating beside it.
        bezier_tube(
            [
                [0.725, 0.425],
                [0.731, 0.47],
                [0.737, 0.515],
                [0.743, 0.555],
            ],
            (0.028, 0.022),
            4,
            FUR_SHADE,
        ),
        // The ear, clear of the skull, then the head, then a muzzle that
        // tapers to the nose.
        disc(0.862, 0.258, 0.028, FUR),
        disc(0.862, 0.262, 0.013, INNER_EAR),
        disc(0.895, 0.325, 0.060, FUR),
        bezier_tube(
            [
                [0.915, 0.340],
                [0.933, 0.344],
                [0.951, 0.348],
                [0.968, 0.352],
            ],
            (0.038, 0.018),
            6,
            FUR,
        ),
        disc(0.912, 0.315, 0.011, DARK),
        disc(0.9155, 0.3115, 0.0035, Color32::WHITE),
        disc(0.977, 0.353, 0.011, DARK),
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
