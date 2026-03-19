/// Chart axis orientation, describing the direction of the X axis.
///
/// The Y axis is always perpendicular to X in the natural direction
/// (bottom-to-top when vertical, left-to-right when horizontal).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    /// X axis: left-to-right, Y axis: bottom-to-top. Standard chart orientation.
    #[default]
    LeftToRight,
    /// X axis: right-to-left, Y axis: bottom-to-top. Mirrored chart.
    RightToLeft,
    /// X axis: top-to-bottom, Y axis: left-to-right. Vertical timeline.
    TopToBottom,
    /// X axis: bottom-to-top, Y axis: left-to-right.
    BottomToTop,
}

impl Orientation {
    /// Returns true if the X axis is horizontal (LeftToRight or RightToLeft).
    pub fn x_is_horizontal(&self) -> bool {
        matches!(self, Self::LeftToRight | Self::RightToLeft)
    }

}
