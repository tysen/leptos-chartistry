use crate::{bounds::Bounds, orientation::Orientation};

/// A projection converts between data and SVG coordinates. SVG has zero in the top left corner. Data coordinates have zero in the bottom left.
#[derive(Clone, Debug, PartialEq)]
pub struct Projection {
    // SVG bounds
    bounds: Bounds,
    // Data offset
    left_x: f64,
    bottom_y: f64,

    x_mult: f64,
    y_mult: f64,

    orientation: Orientation,
}

impl Projection {
    #[cfg(test)]
    pub fn new(bounds: Bounds, range_x: Option<(f64, f64)>, range_y: Option<(f64, f64)>) -> Self {
        Self::with_orientation(bounds, range_x, range_y, Orientation::default())
    }

    pub fn with_orientation(
        bounds: Bounds,
        range_x: Option<(f64, f64)>,
        range_y: Option<(f64, f64)>,
        orientation: Orientation,
    ) -> Self {
        let (left_x, right_x) = range_x.unwrap_or_default();
        let (bottom_y, top_y) = range_y.unwrap_or_default();

        let data_width = right_x - left_x;
        let data_height = top_y - bottom_y;

        // X maps to the screen dimension along the X axis direction
        // Y maps to the perpendicular dimension
        let (x_mult, y_mult) = if orientation.x_is_horizontal() {
            let x_mult = bounds.width() / if data_width == 0.0 { 0.5 } else { data_width };
            let y_mult = bounds.height() / if data_height == 0.0 { 0.5 } else { data_height };
            (x_mult, y_mult)
        } else {
            // X maps to SVG height, Y maps to SVG width
            let x_mult = bounds.height() / if data_width == 0.0 { 0.5 } else { data_width };
            let y_mult = bounds.width() / if data_height == 0.0 { 0.5 } else { data_height };
            (x_mult, y_mult)
        };

        Projection {
            bounds,
            left_x,
            bottom_y,
            x_mult,
            y_mult,
            orientation,
        }
    }

    /// Returns the orientation of this projection.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Converts a data point to SVG view coordinates.
    pub fn position_to_svg(&self, x: f64, y: f64) -> (f64, f64) {
        match self.orientation {
            Orientation::LeftToRight => {
                let svg_x = self.bounds.left_x() + (x - self.left_x) * self.x_mult;
                let svg_y = self.bounds.bottom_y() - (y - self.bottom_y) * self.y_mult;
                (svg_x, svg_y)
            }
            Orientation::RightToLeft => {
                let svg_x = self.bounds.right_x() - (x - self.left_x) * self.x_mult;
                let svg_y = self.bounds.bottom_y() - (y - self.bottom_y) * self.y_mult;
                (svg_x, svg_y)
            }
            Orientation::TopToBottom => {
                let svg_y = self.bounds.top_y() + (x - self.left_x) * self.x_mult;
                let svg_x = self.bounds.left_x() + (y - self.bottom_y) * self.y_mult;
                (svg_x, svg_y)
            }
            Orientation::BottomToTop => {
                let svg_y = self.bounds.bottom_y() - (x - self.left_x) * self.x_mult;
                let svg_x = self.bounds.left_x() + (y - self.bottom_y) * self.y_mult;
                (svg_x, svg_y)
            }
        }
    }

    /// Converts an SVG point to data coordinates.
    pub fn svg_to_position(&self, svg_x: f64, svg_y: f64) -> (f64, f64) {
        match self.orientation {
            Orientation::LeftToRight => {
                let x = self.left_x + (svg_x - self.bounds.left_x()) / self.x_mult;
                let y = self.bottom_y - (svg_y - self.bounds.bottom_y()) / self.y_mult;
                (x, y)
            }
            Orientation::RightToLeft => {
                let x = self.left_x + (self.bounds.right_x() - svg_x) / self.x_mult;
                let y = self.bottom_y - (svg_y - self.bounds.bottom_y()) / self.y_mult;
                (x, y)
            }
            Orientation::TopToBottom => {
                let x = self.left_x + (svg_y - self.bounds.top_y()) / self.x_mult;
                let y = self.bottom_y + (svg_x - self.bounds.left_x()) / self.y_mult;
                (x, y)
            }
            Orientation::BottomToTop => {
                let x = self.left_x + (self.bounds.bottom_y() - svg_y) / self.x_mult;
                let y = self.bottom_y + (svg_x - self.bounds.left_x()) / self.y_mult;
                (x, y)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_coords(p: &Projection, pos: (f64, f64), svg: (f64, f64)) {
        assert_eq!(p.position_to_svg(pos.0, pos.1), (svg.0, svg.1), "to svg");
        assert_eq!(p.svg_to_position(svg.0, svg.1), (pos.0, pos.1), "to pos");
    }

    #[test]
    fn test_left_to_right() {
        let bounds = Bounds::from_points(10.0, 10.0, 90.0, 90.0);
        let p = Projection::new(bounds, Some((0.0, 100.0)), Some((0.0, 100.0)));

        assert_coords(&p, (0.0, 0.0), (10.0, 90.0)); // Bottom left
        assert_coords(&p, (100.0, 0.0), (90.0, 90.0)); // Bottom right
        assert_coords(&p, (0.0, 100.0), (10.0, 10.0)); // Top left
        assert_coords(&p, (100.0, 100.0), (90.0, 10.0)); // Top right
        assert_coords(&p, (50.0, 50.0), (50.0, 50.0)); // Centre
    }

    #[test]
    fn test_right_to_left() {
        let bounds = Bounds::from_points(10.0, 10.0, 90.0, 90.0);
        let p = Projection::with_orientation(
            bounds,
            Some((0.0, 100.0)),
            Some((0.0, 100.0)),
            Orientation::RightToLeft,
        );

        // X is mirrored: X=0 at right, X=100 at left. Y unchanged.
        assert_coords(&p, (0.0, 0.0), (90.0, 90.0)); // Bottom right (X=0 at right)
        assert_coords(&p, (100.0, 0.0), (10.0, 90.0)); // Bottom left (X=100 at left)
        assert_coords(&p, (0.0, 100.0), (90.0, 10.0)); // Top right
        assert_coords(&p, (100.0, 100.0), (10.0, 10.0)); // Top left
        assert_coords(&p, (50.0, 50.0), (50.0, 50.0)); // Centre
    }

    #[test]
    fn test_top_to_bottom() {
        let bounds = Bounds::from_points(10.0, 10.0, 90.0, 90.0);
        let p = Projection::with_orientation(
            bounds,
            Some((0.0, 100.0)),
            Some((0.0, 100.0)),
            Orientation::TopToBottom,
        );

        // X maps to vertical (top-to-bottom), Y maps to horizontal (left-to-right)
        assert_coords(&p, (0.0, 0.0), (10.0, 10.0)); // Top left (X=0 at top, Y=0 at left)
        assert_coords(&p, (100.0, 0.0), (10.0, 90.0)); // Bottom left (X=100 at bottom)
        assert_coords(&p, (0.0, 100.0), (90.0, 10.0)); // Top right (Y=100 at right)
        assert_coords(&p, (100.0, 100.0), (90.0, 90.0)); // Bottom right
        assert_coords(&p, (50.0, 50.0), (50.0, 50.0)); // Centre
    }

    #[test]
    fn test_bottom_to_top() {
        let bounds = Bounds::from_points(10.0, 10.0, 90.0, 90.0);
        let p = Projection::with_orientation(
            bounds,
            Some((0.0, 100.0)),
            Some((0.0, 100.0)),
            Orientation::BottomToTop,
        );

        // X maps to vertical (bottom-to-top), Y maps to horizontal (left-to-right)
        assert_coords(&p, (0.0, 0.0), (10.0, 90.0)); // Bottom left (X=0 at bottom, Y=0 at left)
        assert_coords(&p, (100.0, 0.0), (10.0, 10.0)); // Top left (X=100 at top)
        assert_coords(&p, (0.0, 100.0), (90.0, 90.0)); // Bottom right (Y=100 at right)
        assert_coords(&p, (100.0, 100.0), (90.0, 10.0)); // Top right
        assert_coords(&p, (50.0, 50.0), (50.0, 50.0)); // Centre
    }

    #[test]
    fn test_incl_zero() {
        let bounds = Bounds::from_points(10.0, 10.0, 90.0, 90.0);
        let p = Projection::new(bounds, Some((0.0, 200.0)), Some((0.0, 200.0)));
        assert_coords(&p, (0.0, 0.0), (10.0, 90.0));
        assert_coords(&p, (200.0, 0.0), (90.0, 90.0));
        assert_coords(&p, (0.0, 200.0), (10.0, 10.0));
        assert_coords(&p, (200.0, 200.0), (90.0, 10.0));
        assert_coords(&p, (100.0, 100.0), (50.0, 50.0));
    }

    #[test]
    fn test_projection_zero_range() {
        let bounds = Bounds::from_points(10.0, 10.0, 90.0, 90.0);
        Projection::new(bounds, None, None);
    }

    #[test]
    fn test_partial_eq() {
        let bounds = Bounds::from_points(10.0, 10.0, 90.0, 90.0);
        let p = Projection::new(
            bounds,
            Some((bounds.left_x(), bounds.right_x())),
            Some((bounds.bottom_y(), bounds.top_y())),
        );
        assert_eq!(p, p.clone());
    }
}
