use crate::{
    layout::Layout, projection::Projection, series::UseData, use_watched_node::UseWatchedNode,
    Padding, Tick,
};
use leptos::prelude::*;

/// Pre-layout chart state: debug flags, font metrics, padding, and processed data.
#[derive(Clone)]
#[non_exhaustive]
pub struct PreState<X: Tick, Y: Tick> {
    /// Whether debug mode is enabled.
    pub debug: Signal<bool>,
    /// Font height in pixels.
    pub font_height: Memo<f64>,
    /// Font width of a single monospace character in pixels.
    pub font_width: Memo<f64>,
    /// Chart padding.
    pub padding: Signal<Padding>,
    /// Processed series data.
    pub data: UseData<X, Y>,
}

/// Full chart state after layout: projections, mouse tracking, and pre-layout state.
#[derive(Clone)]
#[non_exhaustive]
pub struct State<X: Tick, Y: Tick> {
    /// Pre-layout state.
    pub pre: PreState<X, Y>,
    /// Computed layout bounds.
    pub layout: Layout,
    /// Projection for the primary (left) Y-axis.
    pub projection_primary: Memo<Projection>,
    /// Projection for the secondary (right) Y-axis.
    pub projection_secondary: Memo<Projection>,

    /// SVG coordinates of the data origin (0, 0) on the primary axis.
    pub svg_zero: Memo<(f64, f64)>,

    /// Mouse page position
    pub mouse_page: Signal<(f64, f64)>,
    /// Mouse page position relative to chart
    pub mouse_chart: Signal<(f64, f64)>,
    /// Mouse over inner chart?
    pub hover_inner: Signal<bool>,
    /// X mouse coord in data position space
    pub hover_position_x: Memo<f64>,
}

impl<X: Tick, Y: Tick> PreState<X, Y> {
    /// Creates a new pre-layout state.
    pub fn new(
        debug: Signal<bool>,
        font_height: Memo<f64>,
        font_width: Memo<f64>,
        padding: Signal<Padding>,
        data: UseData<X, Y>,
    ) -> Self {
        Self {
            debug,
            font_height,
            font_width,
            padding,
            data,
        }
    }
}

impl<X: Tick, Y: Tick> State<X, Y> {
    /// Creates a new chart state from pre-state, layout, and projections.
    pub fn new(
        pre: PreState<X, Y>,
        node: &UseWatchedNode,
        layout: Layout,
        proj_primary: Memo<Projection>,
        proj_secondary: Memo<Projection>,
    ) -> Self {
        // Mouse
        let mouse_chart = node.mouse_chart;
        let hover_inner = node.mouse_hover_inner(layout.inner);

        // Data - use primary projection for hover position
        let hover_position = Memo::new(move |_| {
            let (mouse_x, mouse_y) = mouse_chart.get();
            proj_primary.get().svg_to_position(mouse_x, mouse_y)
        });
        let hover_position_x = Memo::new(move |_| hover_position.get().0);

        Self {
            pre,
            layout,
            projection_primary: proj_primary,
            projection_secondary: proj_secondary,
            svg_zero: Memo::new(move |_| proj_primary.get().position_to_svg(0.0, 0.0)),

            mouse_page: node.mouse_page,
            mouse_chart,
            hover_inner,
            hover_position_x,
        }
    }
}
