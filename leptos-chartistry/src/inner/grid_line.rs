use crate::{
    colors::Color, debug::DebugRect, projection::Projection, series::YAxis, state::State,
    ticks::GeneratedTicks, Tick, TickLabels,
};
use leptos::prelude::*;

/// Default color for grid lines.
pub const GRID_LINE_COLOR: Color = Color::from_rgb(0xEF, 0xF2, 0xFA);

macro_rules! impl_grid_line {
    ($name:ident) => {
        /// Builds a tick-aligned grid line across the inner chart area.
        #[derive(Clone, Debug, PartialEq)]
        #[non_exhaustive]
        pub struct $name<XY: Tick> {
            /// Width of the grid line.
            pub width: RwSignal<f64>,
            /// Color of the grid line.
            pub color: RwSignal<Color>,
            /// Ticks to align the grid line to.
            pub ticks: TickLabels<XY>,
        }

        impl<XY: Tick> $name<XY> {
            /// Creates a new grid line from a set of ticks.
            pub fn from_ticks(ticks: impl Into<TickLabels<XY>>) -> Self {
                Self {
                    ticks: ticks.into(),
                    ..Default::default()
                }
            }

            /// Sets the color of the grid line.
            pub fn with_color(self, color: impl Into<Color>) -> Self {
                self.color.set(color.into());
                self
            }
        }

        impl<XY: Tick> Default for $name<XY> {
            fn default() -> Self {
                Self {
                    width: RwSignal::new(1.0),
                    color: RwSignal::new(GRID_LINE_COLOR),
                    ticks: TickLabels::default(),
                }
            }
        }
    };
}

impl_grid_line!(XGridLine);

/// Builds a tick-aligned grid line across the inner chart area.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct YGridLine<XY: Tick> {
    /// Width of the grid line.
    pub width: RwSignal<f64>,
    /// Color of the grid line.
    pub color: RwSignal<Color>,
    /// Ticks to align the grid line to.
    pub ticks: TickLabels<XY>,
    /// Which Y-axis to align grid lines to. Default is [YAxis::Primary].
    pub axis: YAxis,
}

impl<XY: Tick> YGridLine<XY> {
    /// Creates a new grid line from a set of ticks.
    pub fn from_ticks(ticks: impl Into<TickLabels<XY>>) -> Self {
        Self {
            ticks: ticks.into(),
            ..Default::default()
        }
    }

    /// Sets the color of the grid line.
    pub fn with_color(self, color: impl Into<Color>) -> Self {
        self.color.set(color.into());
        self
    }

    /// Sets which Y-axis the grid lines align to.
    pub fn with_axis(mut self, axis: YAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Creates grid lines aligned to the secondary (right) Y-axis.
    pub fn secondary() -> Self {
        Self {
            axis: YAxis::Secondary,
            ..Default::default()
        }
    }
}

impl<XY: Tick> Default for YGridLine<XY> {
    fn default() -> Self {
        Self {
            width: RwSignal::new(1.0),
            color: RwSignal::new(GRID_LINE_COLOR),
            ticks: TickLabels::default(),
            axis: YAxis::Primary,
        }
    }
}

macro_rules! impl_use_grid_line {
    ($name:ident) => {
        pub struct $name<XY: Tick> {
            width: RwSignal<f64>,
            color: RwSignal<Color>,
            ticks: Memo<GeneratedTicks<XY>>,
        }

        impl<XY: Tick> Clone for $name<XY> {
            fn clone(&self) -> Self {
                Self {
                    width: self.width,
                    color: self.color,
                    ticks: self.ticks,
                }
            }
        }
    };
}

impl_use_grid_line!(UseXGridLine);

pub struct UseYGridLine<XY: Tick> {
    width: RwSignal<f64>,
    color: RwSignal<Color>,
    ticks: Memo<GeneratedTicks<XY>>,
    axis: YAxis,
}

impl<XY: Tick> Clone for UseYGridLine<XY> {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            color: self.color,
            ticks: self.ticks,
            axis: self.axis,
        }
    }
}

impl<X: Tick> XGridLine<X> {
    pub(crate) fn use_horizontal<Y: Tick>(self, state: &State<X, Y>) -> UseXGridLine<X> {
        let inner = state.layout.inner;
        let avail_width = Signal::derive(move || inner.with(|inner| inner.width()));
        UseXGridLine {
            width: self.width,
            color: self.color,
            ticks: self.ticks.generate_horizontal(state.pre.data.range_x, &state.pre, avail_width),
        }
    }
}

impl<Y: Tick> YGridLine<Y> {
    pub(crate) fn use_vertical<X: Tick>(self, state: &State<X, Y>) -> UseYGridLine<Y> {
        let inner = state.layout.inner;
        let avail_height = Signal::derive(move || inner.with(|inner| inner.height()));
        let range = match self.axis {
            YAxis::Primary => state.pre.data.range_y_primary,
            YAxis::Secondary => state.pre.data.range_y_secondary,
        };
        UseYGridLine {
            width: self.width,
            color: self.color,
            ticks: self.ticks.generate_vertical(range, &state.pre, avail_height),
            axis: self.axis,
        }
    }
}

#[component]
pub(super) fn XGridLine<X: Tick, Y: Tick>(
    line: UseXGridLine<X>,
    state: State<X, Y>,
) -> impl IntoView {
    view! {
        <GridLine id="x" ticks=line.ticks proj=state.projection_primary is_x=true
            width=line.width color=line.color state=state />
    }
}

#[component]
pub(super) fn YGridLine<X: Tick, Y: Tick>(
    line: UseYGridLine<Y>,
    state: State<X, Y>,
) -> impl IntoView {
    let proj = match line.axis {
        YAxis::Primary => state.projection_primary,
        YAxis::Secondary => state.projection_secondary,
    };
    view! {
        <GridLine id="y" ticks=line.ticks proj=proj is_x=false
            width=line.width color=line.color state=state />
    }
}

#[component]
fn GridLine<XY: Tick, X: Tick, Y: Tick>(
    id: &'static str,
    ticks: Memo<GeneratedTicks<XY>>,
    proj: Memo<Projection>,
    is_x: bool,
    width: RwSignal<f64>,
    color: RwSignal<Color>,
    state: State<X, Y>,
) -> impl IntoView {
    let debug = state.pre.debug;
    let inner = state.layout.inner;
    let orientation = state.layout.orientation;

    // For X ticks: lines are perpendicular to the X axis (vertical in normal, horizontal in rotated)
    // For Y ticks: lines are perpendicular to the Y axis (horizontal in normal, vertical in rotated)
    // In both cases: `is_x == orientation.x_is_horizontal()` means pos is along svg_x (vertical line),
    // otherwise pos is along svg_y (horizontal line).
    let pos_is_x = is_x == orientation.x_is_horizontal();

    let lines = move || {
        for_ticks(ticks, proj, is_x)
            .into_iter()
            .map(|(pos, label)| {
                let inner = inner.get();
                let (x1, y1, x2, y2) = if pos_is_x {
                    (pos, inner.top_y(), pos, inner.bottom_y())
                } else {
                    (inner.left_x(), pos, inner.right_x(), pos)
                };
                view! {
                    <DebugRect label=format!("grid_line_{}/{}", id, label) debug=debug />
                    <line x1=x1 y1=y1 x2=x2 y2=y2 />
                }
            })
            .collect_view()
    };

    view! {
        <g
            class=format!("_chartistry_grid_line_{}", id)
            stroke=move || color.get().to_string()
            stroke-width=width>
            <DebugRect label=format!("grid_line_{}", id) debug=debug />
            {lines}
        </g>
    }
}

fn for_ticks<XY: Tick>(
    ticks: Memo<GeneratedTicks<XY>>,
    proj: Memo<Projection>,
    is_x: bool,
) -> Vec<(f64, String)> {
    ticks.with(move |ticks| {
        let proj = proj.get();
        let orientation = proj.orientation();
        ticks
            .ticks
            .iter()
            .map(|tick| {
                let label = ticks.state.format(tick);
                let tick_pos = tick.position();
                // Get the SVG position for this tick
                // For X ticks, we want the position where X = tick_pos
                // For Y ticks, we want the position where Y = tick_pos
                let svg_pos = if is_x {
                    proj.position_to_svg(tick_pos, 0.0)
                } else {
                    proj.position_to_svg(0.0, tick_pos)
                };
                // When is_x matches x_is_horizontal, the tick aligns with svg_x; otherwise svg_y
                let pos = if is_x == orientation.x_is_horizontal() { svg_pos.0 } else { svg_pos.1 };
                (pos, label)
            })
            .collect::<Vec<_>>()
    })
}
