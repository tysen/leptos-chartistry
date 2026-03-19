/// Line interpolation. This is used to determine how to draw the line between points.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum Interpolation {
    /// Linear interpolation draws a straight line between points. The simplest of methods.
    Linear,
    /// Step interpolation only uses horizontal and vertical lines to connect two points.
    Step(Step),
    /// Cubic monotone interpolation smooths the line between points. Avoids spurious oscillations.[^Steffen]
    ///
    /// [^Steffen]: Steffen, M., "A simple method for monotonic interpolation in one dimension.", Astronomy and Astrophysics, vol. 239, pp. 443–450, 1990.
    #[default]
    Monotone,
}

/// Step interpolation only uses horizontal and vertical lines to connect two points. We have a choice of where to put the "corner" of the step.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum Step {
    /// Moves across the horizontal plane first then the vertical.
    #[default]
    Horizontal,
    /// Moves midway across the horizontal plane first, then all of the vertical, then the rest of the horizontal. When chained with other steps, creates a single step but could make the data point less obvious.
    HorizontalMiddle,
    /// Moves across the vertical plane first then the horizontal.
    Vertical,
    /// Similar to [Step::HorizontalMiddle] but moves midway across the vertical plane first, then all of the horizontal, then the rest of the vertical.
    VerticalMiddle,
}

impl std::str::FromStr for Interpolation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linear" => Ok(Self::Linear),
            "step-horizontal" => Ok(Self::Step(Step::Horizontal)),
            "step-horizontal-middle" => Ok(Self::Step(Step::HorizontalMiddle)),
            "step-vertical" => Ok(Self::Step(Step::Vertical)),
            "step-vertical-middle" => Ok(Self::Step(Step::VerticalMiddle)),
            "monotone" => Ok(Self::Monotone),
            _ => Err(format!("unknown line interpolation: `{}`", s)),
        }
    }
}

impl std::fmt::Display for Interpolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linear => write!(f, "linear"),
            Self::Step(Step::Horizontal) => write!(f, "step-horizontal"),
            Self::Step(Step::HorizontalMiddle) => write!(f, "step-horizontal-middle"),
            Self::Step(Step::Vertical) => write!(f, "step-vertical"),
            Self::Step(Step::VerticalMiddle) => write!(f, "step-vertical-middle"),
            Self::Monotone => write!(f, "monotone"),
        }
    }
}

impl From<Step> for Interpolation {
    fn from(step: Step) -> Self {
        Self::Step(step)
    }
}

impl Interpolation {
    /// Generate an SVG path string from points in SVG coordinates.
    ///
    /// `x_is_horizontal` indicates whether the independent (monotonically increasing)
    /// coordinate is svg_x (true, normal mode) or svg_y (false, vertical orientation).
    /// This affects interpolation algorithms that assume monotonic x values.
    pub(super) fn path(self, points: &[(f64, f64)], x_is_horizontal: bool) -> String {
        match self {
            Self::Linear => linear(points),
            Self::Step(step) => step.path(points, x_is_horizontal),
            Self::Monotone => monotone(points, x_is_horizontal),
        }
    }
}

fn linear(points: &[(f64, f64)]) -> String {
    let mut need_move = true;
    points
        .iter()
        .map(|(x, y)| {
            if x.is_nan() || y.is_nan() {
                need_move = true;
                "".to_string()
            } else if need_move {
                need_move = false;
                format!("M {} {} ", x, y)
            } else {
                format!("L {} {} ", x, y)
            }
        })
        .collect::<String>()
}

impl Step {
    fn path(self, points: &[(f64, f64)], x_is_horizontal: bool) -> String {
        // Map SVG coordinates to (independent, dependent) based on orientation.
        // When X is horizontal: independent = svg_x (H command), dependent = svg_y (V command).
        // When X is vertical: independent = svg_y (V command), dependent = svg_x (H command).
        // We swap coordinates so the match arms always use H for independent, V for dependent.
        let (h, v) = if x_is_horizontal { ("H", "V") } else { ("V", "H") };
        let mut prev: Option<(f64, f64)> = None;
        points
            .iter()
            .map(|&(sx, sy)| {
                if sx.is_nan() || sy.is_nan() {
                    prev = None;
                    "".to_string()
                } else if let Some((prev_sx, prev_sy)) = prev {
                    prev = Some((sx, sy));
                    // Map to (independent, dependent) SVG values
                    let (ind, dep, prev_ind, prev_dep) = if x_is_horizontal {
                        (sx, sy, prev_sx, prev_sy)
                    } else {
                        (sy, sx, prev_sy, prev_sx)
                    };
                    match self {
                        Self::Horizontal => format!("{h} {ind} {v} {dep} "),
                        Self::HorizontalMiddle => {
                            let mid = (ind + prev_ind) / 2.0;
                            format!("{h} {mid} {v} {dep} {h} {ind} ")
                        }
                        Self::Vertical => format!("{v} {dep} {h} {ind} "),
                        Self::VerticalMiddle => {
                            let mid = (dep + prev_dep) / 2.0;
                            format!("{v} {mid} {h} {ind} {v} {dep} ")
                        }
                    }
                } else {
                    prev = Some((sx, sy));
                    format!("M {} {} ", sx, sy)
                }
            })
            .collect::<String>()
    }
}

/*
    Implementation from "A simple method for monotonic interpolation in one dimension". [^Steffen]

    In Fortran:
        y1(i) = (sign(1.0, s[i-1]) + sign(1.0, s[i])) * min(abs(s[i-1]), abs(s[i]), 0.5 * abs(p[i]))
    Where:
        s[i] = (y[i+1] - y[i]) / (x[i+1] - x[i])
        p[i] = (s[i-1]h[i] + s[i]h[i-1]) / (h[i-1] + h[i])
        h[i] = x[i+1] - x[i]

    In Rust:
        y(i) = (s[i-1].signum() + s[i].signum()) * s[i-1].abs().min(s[i].abs()).min(0.5 * p[i].abs())
*/

/// Monotone interpolation parameterized by which SVG axis is the independent
/// (monotonically increasing) variable. When `x_is_horizontal`, svg_x is
/// independent; otherwise svg_y is independent.
fn monotone(points: &[(f64, f64)], x_is_horizontal: bool) -> String {
    let mut path = String::with_capacity(points.len());
    for i in 0..points.len() {
        let prev = get_or_nan(points, i.checked_sub(1));
        let cur = points[i];
        let next = get_or_nan(points, i.checked_add(1));

        // Select independent/dependent based on orientation
        let (ind_prev, dep_prev, ind, dep, ind_next, dep_next) = if x_is_horizontal {
            (prev.0, prev.1, cur.0, cur.1, next.0, next.1)
        } else {
            (prev.1, prev.0, cur.1, cur.0, next.1, next.0)
        };

        let cmd = if ind.is_nan() || dep.is_nan() {
            "".to_string()
        } else if ind_prev.is_nan() || dep_prev.is_nan() {
            format!("M {},{} ", cur.0, cur.1)
        } else if ind_next.is_nan() || dep_next.is_nan() {
            format!("L {},{} ", cur.0, cur.1)
        } else {
            let t = tangent(ind_prev, ind, ind_next, dep_prev, dep, dep_next);
            let d_ind = (ind - ind_prev) / 3.0;
            let ind_c = ind - d_ind;
            let dep_c = dep - d_ind * t;
            // Reconstruct (svg_x, svg_y)
            let (xc, yc) = if x_is_horizontal {
                (ind_c, dep_c)
            } else {
                (dep_c, ind_c)
            };
            format!("S {xc},{yc} {},{} ", cur.0, cur.1)
        };
        path.push_str(&cmd);
    }
    path
}

fn get_or_nan(points: &[(f64, f64)], i: Option<usize>) -> (f64, f64) {
    i.and_then(|i| points.get(i).copied())
        .unwrap_or((f64::NAN, f64::NAN))
}

fn slope(x: f64, y: f64, x_next: f64, y_next: f64) -> f64 {
    (y_next - y) / (x_next - x)
}

fn tangent(x_prev: f64, x: f64, x_next: f64, y_prev: f64, y: f64, y_next: f64) -> f64 {
    let slope_prev = slope(x_prev, y_prev, x, y);
    let slope = slope(x, y, x_next, y_next);
    // Parabola
    let dist_prev = x - x_prev;
    let dist = x_next - x;
    let para = (slope_prev * dist + slope * dist_prev) / (dist_prev + dist);
    // Tangent: limit to min of both slopes and half the parabolic interpolant
    (slope_prev.signum() + slope.signum()) * slope_prev.abs().min(slope.abs()).min(0.5 * para.abs())
}
