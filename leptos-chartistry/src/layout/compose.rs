use super::{AxisEdges, EdgeLayout, UseLayout};
use crate::{
    aspect_ratio::KnownAspectRatio,
    bounds::Bounds,
    edge::Edge,
    orientation::Orientation,
    series::YAxis,
    state::{PreState, State},
    Tick,
};
use leptos::prelude::*;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Layout {
    pub outer: Memo<Bounds>,
    pub top: Vec<Memo<Bounds>>,
    pub right: Vec<Memo<Bounds>>,
    pub bottom: Vec<Memo<Bounds>>,
    pub left: Vec<Memo<Bounds>>,
    pub inner: Memo<Bounds>,
    /// Width per X data point (horizontal spacing, meaningful when X is horizontal).
    pub x_width: Memo<f64>,
    /// Height per X data point (vertical spacing, meaningful when X is vertical).
    pub y_height: Memo<f64>,
    /// The orientation of the chart.
    pub orientation: Orientation,
}

#[derive(Clone)]
pub struct DeferredRender {
    edge: Edge,
    bounds: Memo<Bounds>,
    layout: UseLayout,
}

impl DeferredRender {
    pub fn render<X: Tick, Y: Tick>(self, state: State<X, Y>) -> impl IntoView {
        self.layout.render(self.edge, self.bounds, state)
    }
}

impl Layout {
    /// Composes a layout from semantic axis edges, mapping them to physical edges based on orientation.
    pub fn compose<X: Tick, Y: Tick>(
        x_axis: &AxisEdges<X>,
        y_axis: &AxisEdges<Y>,
        aspect_ratio: Memo<KnownAspectRatio>,
        state: &PreState<X, Y>,
        orientation: Orientation,
    ) -> (Layout, Vec<DeferredRender>) {
        if orientation.x_is_horizontal() {
            // X on horizontal edges (top/bottom), Y on vertical edges (left/right)
            compose_x_horizontal(x_axis, y_axis, aspect_ratio, state, orientation)
        } else {
            // X on vertical edges (left/right), Y on horizontal edges (top/bottom)
            compose_x_vertical(x_axis, y_axis, aspect_ratio, state, orientation)
        }
    }
}

/// Normal orientation: X data on top/bottom, Y data on left/right.
fn compose_x_horizontal<X: Tick, Y: Tick>(
    x_axis: &AxisEdges<X>,
    y_axis: &AxisEdges<Y>,
    aspect_ratio: Memo<KnownAspectRatio>,
    state: &PreState<X, Y>,
    orientation: Orientation,
) -> (Layout, Vec<DeferredRender>) {
    // Physical edges: top=x_axis.start, bottom=x_axis.end, left=y_axis.start, right=y_axis.end
    let top = &x_axis.start;
    let bottom = &x_axis.end;
    let left = &y_axis.start;
    let right = &y_axis.end;

    // Horizontal edges have fixed heights
    let top_heights = collect_heights(top, state);
    let top_height = sum_sizes(top_heights.clone());
    let bottom_heights = collect_heights(bottom, state);
    let bottom_height = sum_sizes(bottom_heights.clone());
    let inner_height =
        KnownAspectRatio::inner_height_signal(aspect_ratio, top_height, bottom_height);

    // Vertical edges have variable widths
    let (left_widths, left_layouts) = use_vertical_y(left, state, inner_height, YAxis::Primary);
    let left_width = sum_sizes(left_widths.clone());
    let (right_widths, right_layouts) =
        use_vertical_y(right, state, inner_height, YAxis::Secondary);
    let right_width = sum_sizes(right_widths.clone());
    let avail_width =
        KnownAspectRatio::inner_width_signal(aspect_ratio, left_width, right_width);

    let (layout, mut deferred) = build_layout(
        aspect_ratio,
        state,
        orientation,
        top_heights,
        top_height,
        bottom_heights,
        bottom_height,
        left_widths,
        left_width,
        right_widths,
        right_width,
        inner_height,
        avail_width,
    );

    // Build deferred renders
    let left_renders = vertical_renders(Edge::Left, &layout.left, left_layouts);
    let right_renders = vertical_renders(Edge::Right, &layout.right, right_layouts);
    let top_renders = horizontal_x_renders(Edge::Top, &layout.top, top, state, avail_width);
    let bottom_renders =
        horizontal_x_renders(Edge::Bottom, &layout.bottom, bottom, state, avail_width);

    deferred.extend(left_renders);
    deferred.extend(right_renders);
    deferred.extend(top_renders);
    deferred.extend(bottom_renders);

    (layout, deferred)
}

/// Rotated orientation: Y data on top/bottom, X data on left/right.
fn compose_x_vertical<X: Tick, Y: Tick>(
    x_axis: &AxisEdges<X>,
    y_axis: &AxisEdges<Y>,
    aspect_ratio: Memo<KnownAspectRatio>,
    state: &PreState<X, Y>,
    orientation: Orientation,
) -> (Layout, Vec<DeferredRender>) {
    // Physical edges: top=y_axis.start, bottom=y_axis.end, left=x_axis.start, right=x_axis.end
    let top = &y_axis.start;
    let bottom = &y_axis.end;
    let left = &x_axis.start;
    let right = &x_axis.end;

    // Horizontal edges (Y data) have fixed heights
    let top_heights = collect_heights(top, state);
    let top_height = sum_sizes(top_heights.clone());
    let bottom_heights = collect_heights(bottom, state);
    let bottom_height = sum_sizes(bottom_heights.clone());
    let inner_height =
        KnownAspectRatio::inner_height_signal(aspect_ratio, top_height, bottom_height);

    // Vertical edges (X data) have variable widths
    let (left_widths, left_layouts) = use_vertical_x(left, state, inner_height);
    let left_width = sum_sizes(left_widths.clone());
    let (right_widths, right_layouts) = use_vertical_x(right, state, inner_height);
    let right_width = sum_sizes(right_widths.clone());
    let avail_width =
        KnownAspectRatio::inner_width_signal(aspect_ratio, left_width, right_width);

    let (layout, mut deferred) = build_layout(
        aspect_ratio,
        state,
        orientation,
        top_heights,
        top_height,
        bottom_heights,
        bottom_height,
        left_widths,
        left_width,
        right_widths,
        right_width,
        inner_height,
        avail_width,
    );

    // Build deferred renders
    let left_renders = vertical_renders(Edge::Left, &layout.left, left_layouts);
    let right_renders = vertical_renders(Edge::Right, &layout.right, right_layouts);
    let top_renders =
        horizontal_y_renders(Edge::Top, &layout.top, top, state, avail_width, YAxis::Primary);
    let bottom_renders = horizontal_y_renders(
        Edge::Bottom,
        &layout.bottom,
        bottom,
        state,
        avail_width,
        YAxis::Secondary,
    );

    deferred.extend(left_renders);
    deferred.extend(right_renders);
    deferred.extend(top_renders);
    deferred.extend(bottom_renders);

    (layout, deferred)
}

/// Build the Layout struct and bounds (shared between both orientation paths).
#[allow(clippy::too_many_arguments)]
fn build_layout<X: Tick, Y: Tick>(
    _aspect_ratio: Memo<KnownAspectRatio>,
    state: &PreState<X, Y>,
    orientation: Orientation,
    top_heights: Vec<Signal<f64>>,
    top_height: Memo<f64>,
    bottom_heights: Vec<Signal<f64>>,
    bottom_height: Memo<f64>,
    left_widths: Vec<Signal<f64>>,
    left_width: Memo<f64>,
    right_widths: Vec<Signal<f64>>,
    right_width: Memo<f64>,
    inner_height: Memo<f64>,
    avail_width: Memo<f64>,
) -> (Layout, Vec<DeferredRender>) {
    // Bounds
    let outer = Memo::new(move |_| {
        Bounds::new(
            left_width.get() + avail_width.get() + right_width.get(),
            top_height.get() + inner_height.get() + bottom_height.get(),
        )
    });
    let inner = Memo::new(move |_| {
        outer.get().shrink(
            top_height.get(),
            right_width.get(),
            bottom_height.get(),
            left_width.get(),
        )
    });

    // Edge bounds
    let top_bounds = Memo::new(move |_| {
        let i = inner.get();
        Bounds::from_points(i.left_x(), outer.get().top_y(), i.right_x(), i.top_y())
    });
    let right_bounds = Memo::new(move |_| {
        let i = inner.get();
        Bounds::from_points(i.right_x(), i.top_y(), outer.get().right_x(), i.bottom_y())
    });
    let bottom_bounds = Memo::new(move |_| {
        let i = inner.get();
        let bottom_y = outer.get().bottom_y();
        Bounds::from_points(i.left_x(), i.bottom_y(), i.right_x(), bottom_y)
    });
    let left_bounds = Memo::new(move |_| {
        let i = inner.get();
        Bounds::from_points(outer.get().left_x(), i.top_y(), i.left_x(), i.bottom_y())
    });

    // Size per data point along each axis direction.
    // X runs along SVG width when horizontal, SVG height when vertical.
    let data_len = state.data.len;
    let x_width = Memo::new(move |_| {
        let size = if orientation.x_is_horizontal() {
            inner.get().width()
        } else {
            inner.get().height()
        };
        size / data_len.get() as f64
    });
    let y_height = Memo::new(move |_| {
        let size = if orientation.x_is_horizontal() {
            inner.get().height()
        } else {
            inner.get().width()
        };
        size / data_len.get() as f64
    });

    let layout = Layout {
        outer,
        top: option_bounds(Edge::Top, top_bounds, top_heights),
        right: option_bounds(Edge::Right, right_bounds, right_widths),
        bottom: option_bounds(Edge::Bottom, bottom_bounds, bottom_heights),
        left: option_bounds(Edge::Left, left_bounds, left_widths),
        inner,
        x_width,
        y_height,
        orientation,
    };

    (layout, Vec::new())
}

fn collect_heights<XY: Tick, X: Tick, Y: Tick>(
    items: &[EdgeLayout<XY>],
    state: &PreState<X, Y>,
) -> Vec<Signal<f64>> {
    items.iter().map(|c| c.fixed_height(state)).collect()
}

// --- Helper functions for X-axis data on horizontal edges ---

fn horizontal_x_renders<X: Tick, Y: Tick>(
    edge: Edge,
    bounds: &[Memo<Bounds>],
    items: &[EdgeLayout<X>],
    state: &PreState<X, Y>,
    avail_width: Memo<f64>,
) -> Vec<DeferredRender> {
    items
        .iter()
        .enumerate()
        .map(|(index, opt)| DeferredRender {
            edge,
            bounds: bounds[index],
            layout: opt.to_horizontal_use_x(state, avail_width),
        })
        .collect()
}

// --- Helper functions for Y-axis data on horizontal edges ---

fn horizontal_y_renders<X: Tick, Y: Tick>(
    edge: Edge,
    bounds: &[Memo<Bounds>],
    items: &[EdgeLayout<Y>],
    state: &PreState<X, Y>,
    avail_width: Memo<f64>,
    axis: YAxis,
) -> Vec<DeferredRender> {
    items
        .iter()
        .enumerate()
        .map(|(index, opt)| DeferredRender {
            edge,
            bounds: bounds[index],
            layout: opt.to_horizontal_use_y(state, avail_width, axis),
        })
        .collect()
}

// --- Helper functions for Y-axis data on vertical edges ---

fn use_vertical_y<X: Tick, Y: Tick>(
    items: &[EdgeLayout<Y>],
    state: &PreState<X, Y>,
    avail_height: Memo<f64>,
    axis: YAxis,
) -> (Vec<Signal<f64>>, Vec<UseLayout>) {
    items
        .iter()
        .map(|c| {
            let vert = c.to_vertical_use_y(state, avail_height, axis);
            (vert.width, vert.layout)
        })
        .unzip()
}

// --- Helper functions for X-axis data on vertical edges ---

fn use_vertical_x<X: Tick, Y: Tick>(
    items: &[EdgeLayout<X>],
    state: &PreState<X, Y>,
    avail_height: Memo<f64>,
) -> (Vec<Signal<f64>>, Vec<UseLayout>) {
    items
        .iter()
        .map(|c| {
            let vert = c.to_vertical_use_x(state, avail_height);
            (vert.width, vert.layout)
        })
        .unzip()
}

// --- Shared helpers ---

fn vertical_renders(
    edge: Edge,
    bounds: &[Memo<Bounds>],
    layouts: Vec<UseLayout>,
) -> Vec<DeferredRender> {
    layouts
        .into_iter()
        .enumerate()
        .map(|(index, layout)| DeferredRender {
            edge,
            bounds: bounds[index],
            layout,
        })
        .collect()
}

fn sum_sizes(sizes: Vec<Signal<f64>>) -> Memo<f64> {
    Memo::new(move |_| sizes.iter().map(|opt| opt.get()).sum::<f64>())
}

fn option_bounds(edge: Edge, outer: Memo<Bounds>, sizes: Vec<Signal<f64>>) -> Vec<Memo<Bounds>> {
    let mut seen = Vec::<Signal<f64>>::with_capacity(sizes.len());
    sizes
        .into_iter()
        .map(|size| {
            let prev = seen.clone();
            seen.push(size);
            Memo::new(move |_| {
                // Proximal "nearest" and distal "furthest" are distances from the inner edge
                let proximal = prev.iter().map(|s| s.get()).sum::<f64>();
                let distal = proximal + size.get();
                let outer = outer.get();
                let width = outer.width();
                let height = outer.height();
                match edge {
                    Edge::Top => outer.shrink(height - distal, 0.0, proximal, 0.0),
                    Edge::Bottom => outer.shrink(proximal, 0.0, height - distal, 0.0),
                    Edge::Left => outer.shrink(0.0, proximal, 0.0, width - distal),
                    Edge::Right => outer.shrink(0.0, width - distal, 0.0, proximal),
                }
            })
        })
        .collect::<Vec<_>>()
}
