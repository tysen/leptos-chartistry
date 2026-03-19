mod compose;
pub mod legend;
pub mod rotated_label;
pub mod tick_labels;

pub use compose::Layout;

use crate::{
    bounds::Bounds,
    edge::Edge,
    series::YAxis,
    state::{PreState, State},
    Tick,
};
use tick_labels::DataAxis;
use leptos::prelude::*;

/// All possible layout options for an edge of a [Chart](crate::Chart). See [IntoEdge](trait@IntoEdge) for details.
#[derive(Clone)]
#[non_exhaustive]
pub enum EdgeLayout<XY: Tick> {
    /// Legend. See [legend](struct@legend::Legend) for details.
    Legend(legend::Legend),
    /// Rotated label. See [rotated_label](struct@rotated_label::RotatedLabel) for details.
    RotatedLabel(rotated_label::RotatedLabel),
    /// Tick labels. See [tick_labels](struct@tick_labels::TickLabels) for details.
    TickLabels(tick_labels::TickLabels<XY>),
}

/// Edge components for one axis, placed on two opposite sides of the chart.
///
/// For the X axis: `start` and `end` are the two edges perpendicular to the X axis.
/// For the Y axis: `start` maps to `YAxis::Primary`, `end` maps to `YAxis::Secondary`.
///
/// Which physical edges these map to depends on the chart's [Orientation](crate::Orientation).
#[derive(Clone)]
pub struct AxisEdges<XY: Tick> {
    /// Components on the "start" side of the axis.
    pub start: Vec<EdgeLayout<XY>>,
    /// Components on the "end" side of the axis.
    pub end: Vec<EdgeLayout<XY>>,
}

impl<XY: Tick> Default for AxisEdges<XY> {
    fn default() -> Self {
        Self {
            start: Vec::new(),
            end: Vec::new(),
        }
    }
}

impl<XY: Tick> AxisEdges<XY> {
    /// Creates empty axis edges.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the start side components.
    pub fn start(mut self, items: impl Into<Vec<EdgeLayout<XY>>>) -> Self {
        self.start = items.into();
        self
    }

    /// Sets the end side components.
    pub fn end(mut self, items: impl Into<Vec<EdgeLayout<XY>>>) -> Self {
        self.end = items.into();
        self
    }
}

struct UseVerticalLayout {
    width: Signal<f64>,
    layout: UseLayout,
}

#[derive(Clone)]
enum UseLayout {
    Legend(legend::Legend),
    RotatedLabel(rotated_label::RotatedLabel),
    TickLabels(tick_labels::UseTickLabels),
}

impl UseLayout {
    fn render<X: Tick, Y: Tick>(
        self,
        edge: Edge,
        bounds: Memo<Bounds>,
        state: State<X, Y>,
    ) -> impl IntoView {
        match self {
            Self::Legend(inner) => view! {<legend::Legend legend=inner edge=edge bounds=bounds state=state />}.into_any(),
            Self::RotatedLabel(inner) => view! {<rotated_label::RotatedLabel label=inner edge=edge bounds=bounds state=state />}.into_any(),
            Self::TickLabels(inner) => view! {<tick_labels::TickLabels ticks=inner edge=edge bounds=bounds state=state />}.into_any(),
        }
    }
}

impl<XY: Tick> EdgeLayout<XY> {
    fn fixed_height<X: Tick, Y: Tick>(&self, state: &PreState<X, Y>) -> Signal<f64> {
        match self {
            Self::Legend(inner) => inner.fixed_height(state),
            Self::RotatedLabel(inner) => inner.fixed_height(state),
            Self::TickLabels(inner) => inner.fixed_height(state),
        }
    }
}

// Methods for X-axis data on physical edges
impl<X: Tick> EdgeLayout<X> {
    fn to_horizontal_use_x<Y: Tick>(
        &self,
        state: &PreState<X, Y>,
        avail_width: Memo<f64>,
    ) -> UseLayout {
        match self {
            Self::Legend(inner) => inner.to_horizontal_use(),
            Self::RotatedLabel(inner) => inner.to_horizontal_use(),
            Self::TickLabels(inner) => {
                inner.to_horizontal_use(state.data.range_x, state, avail_width, DataAxis::X)
            }
        }
    }

    fn to_vertical_use_x<Y: Tick>(
        &self,
        state: &PreState<X, Y>,
        avail_height: Memo<f64>,
    ) -> UseVerticalLayout {
        match self {
            Self::Legend(inner) => inner.to_vertical_use(state),
            Self::RotatedLabel(inner) => inner.to_vertical_use(state),
            Self::TickLabels(inner) => {
                inner.to_vertical_use(state.data.range_x, state, avail_height, DataAxis::X)
            }
        }
    }
}

// Methods for Y-axis data on physical edges
impl<Y: Tick> EdgeLayout<Y> {
    fn to_vertical_use_y<X: Tick>(
        &self,
        state: &PreState<X, Y>,
        avail_height: Memo<f64>,
        axis: YAxis,
    ) -> UseVerticalLayout {
        match self {
            Self::Legend(inner) => inner.to_vertical_use(state),
            Self::RotatedLabel(inner) => inner.to_vertical_use(state),
            Self::TickLabels(inner) => {
                let range = Self::range_y(state, axis);
                inner.to_vertical_use(range, state, avail_height, DataAxis::Y)
            }
        }
    }

    fn to_horizontal_use_y<X: Tick>(
        &self,
        state: &PreState<X, Y>,
        avail_width: Memo<f64>,
        axis: YAxis,
    ) -> UseLayout {
        match self {
            Self::Legend(inner) => inner.to_horizontal_use(),
            Self::RotatedLabel(inner) => inner.to_horizontal_use(),
            Self::TickLabels(inner) => {
                let range = Self::range_y(state, axis);
                inner.to_horizontal_use(range, state, avail_width, DataAxis::Y)
            }
        }
    }

    fn range_y<X: Tick>(state: &PreState<X, Y>, axis: YAxis) -> Memo<crate::series::Range<Y>> {
        match axis {
            YAxis::Primary => state.data.range_y_primary,
            YAxis::Secondary => state.data.range_y_secondary,
        }
    }
}

/// Convert a type (e.g., a [rotated label](struct@rotated_label::RotatedLabel)) into an edge layout for use with [Chart](crate::Chart).
pub trait IntoEdge<XY: Tick> {
    /// Create an edge layout from the type. See [IntoEdge](trait@IntoEdge) for details.
    fn into_edge(self) -> EdgeLayout<XY>;
}

macro_rules! impl_into_edge {
    ($ty:ty, $enum:ident) => {
        impl<XY: Tick> IntoEdge<XY> for $ty {
            fn into_edge(self) -> EdgeLayout<XY> {
                EdgeLayout::$enum(self)
            }
        }

        impl<XY: Tick> From<$ty> for EdgeLayout<XY> {
            fn from(inner: $ty) -> Self {
                inner.into_edge()
            }
        }

        impl<XY: Tick> From<$ty> for Vec<EdgeLayout<XY>> {
            fn from(inner: $ty) -> Self {
                vec![inner.into_edge()]
            }
        }
    };
}
impl_into_edge!(legend::Legend, Legend);
impl_into_edge!(rotated_label::RotatedLabel, RotatedLabel);
impl_into_edge!(tick_labels::TickLabels<XY>, TickLabels);
