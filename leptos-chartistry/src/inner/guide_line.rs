use crate::{bounds::Bounds, colors::Color, debug::DebugRect, state::State, Tick};
use leptos::prelude::*;
use std::str::FromStr;

/// Default color for guide lines.
pub const GUIDE_LINE_COLOR: Color = Color::from_rgb(0x9A, 0x9A, 0x9A);

macro_rules! impl_guide_line {
    ($name:ident) => {
        /// Builds a mouse guide line. Aligned over the mouse position or nearest data.
        #[derive(Clone, Debug, PartialEq)]
        #[non_exhaustive]
        pub struct $name {
            /// Alignment of the guide line.
            pub align: RwSignal<AlignOver>,
            /// Width of the guide line.
            pub width: RwSignal<f64>,
            /// Color of the guide line.
            pub color: RwSignal<Color>,
        }

        impl $name {
            fn new(align: AlignOver) -> Self {
                Self {
                    align: RwSignal::new(align),
                    width: RwSignal::new(1.0),
                    color: RwSignal::new(GUIDE_LINE_COLOR),
                }
            }

            /// Creates a new guide line aligned over the mouse position.
            pub fn over_mouse() -> Self {
                Self::new(AlignOver::Mouse)
            }

            /// Creates a new guide line aligned over the nearest data.
            pub fn over_data() -> Self {
                Self::new(AlignOver::Data)
            }

            /// Sets the color of the guide line.
            pub fn with_color(self, color: impl Into<Color>) -> Self {
                self.color.set(color.into());
                self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(AlignOver::default())
            }
        }
    };
}

impl_guide_line!(XGuideLine);
impl_guide_line!(YGuideLine);

/// Align over mouse or nearest data.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum AlignOver {
    /// Align over the mouse position.
    #[default]
    Mouse,
    /// Align over the nearest data. Creates a "snap to data" effect.
    Data,
}

#[derive(Clone)]
pub struct UseXGuideLine(XGuideLine);

#[derive(Clone)]
pub struct UseYGuideLine(YGuideLine);

impl XGuideLine {
    pub(crate) fn use_horizontal(self) -> UseXGuideLine {
        UseXGuideLine(self)
    }
}

impl YGuideLine {
    pub(crate) fn use_vertical(self) -> UseYGuideLine {
        UseYGuideLine(self)
    }
}

impl std::fmt::Display for AlignOver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlignOver::Mouse => write!(f, "mouse"),
            AlignOver::Data => write!(f, "data"),
        }
    }
}

impl FromStr for AlignOver {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mouse" => Ok(AlignOver::Mouse),
            "data" => Ok(AlignOver::Data),
            _ => Err(format!("invalid align over: `{}`", s)),
        }
    }
}

#[component]
pub(super) fn XGuideLine<X: Tick, Y: Tick>(
    line: UseXGuideLine,
    state: State<X, Y>,
) -> impl IntoView {
    let line = line.0;
    let inner = state.layout.inner;
    let mouse_chart = state.mouse_chart;
    let orientation = state.layout.orientation;

    // Data alignment - get nearest X position in data space
    let nearest_pos_x = state.pre.data.nearest_position_x(state.hover_position_x);
    // Convert to SVG coordinates
    let nearest_svg = Memo::new(move |_| {
        nearest_pos_x
            .get()
            .map(|pos_x| state.projection_primary.get().position_to_svg(pos_x, 0.0))
    });

    let pos = Signal::derive(move || {
        let (mouse_x, mouse_y) = mouse_chart.get();
        let inner = inner.get();

        if orientation.x_is_horizontal() {
            // X guide line is vertical (shows X position)
            let x = match line.align.get() {
                AlignOver::Data => nearest_svg.get().map(|(x, _)| x).unwrap_or(mouse_x),
                AlignOver::Mouse => mouse_x,
            };
            Bounds::from_points(x, inner.top_y(), x, inner.bottom_y())
        } else {
            // X guide line is horizontal (X is now vertical axis)
            let y = match line.align.get() {
                AlignOver::Data => nearest_svg.get().map(|(_, y)| y).unwrap_or(mouse_y),
                AlignOver::Mouse => mouse_y,
            };
            Bounds::from_points(inner.left_x(), y, inner.right_x(), y)
        }
    });

    view! {
        <GuideLine id="x" width=line.width color=line.color state=state pos=pos />
    }
}

#[component]
pub(super) fn YGuideLine<X: Tick, Y: Tick>(
    line: UseYGuideLine,
    state: State<X, Y>,
) -> impl IntoView {
    let line = line.0;
    let inner = state.layout.inner;
    let mouse_chart = state.mouse_chart;
    let orientation = state.layout.orientation;

    // TODO align over
    let pos = Signal::derive(move || {
        let (mouse_x, mouse_y) = mouse_chart.get();
        let inner = inner.get();

        if orientation.x_is_horizontal() {
            // Y guide line is horizontal (Y is vertical axis)
            Bounds::from_points(inner.left_x(), mouse_y, inner.right_x(), mouse_y)
        } else {
            // Y guide line is vertical (Y is now horizontal axis)
            Bounds::from_points(mouse_x, inner.top_y(), mouse_x, inner.bottom_y())
        }
    });
    view! {
        <GuideLine id="y" width=line.width color=line.color state=state pos=pos />
    }
}

#[component]
fn GuideLine<X: Tick, Y: Tick>(
    id: &'static str,
    width: RwSignal<f64>,
    color: RwSignal<Color>,
    state: State<X, Y>,
    pos: Signal<Bounds>,
) -> impl IntoView {
    let debug = state.pre.debug;
    let hover_inner = state.hover_inner;

    let x1 = Memo::new(move |_| pos.get().left_x());
    let y1 = Memo::new(move |_| pos.get().top_y());
    let x2 = Memo::new(move |_| pos.get().right_x());
    let y2 = Memo::new(move |_| pos.get().bottom_y());

    // Don't render if any of the coordinates are NaN i.e., no data
    let have_data = Signal::derive(move || {
        !(x1.get().is_nan() || y1.get().is_nan() || x2.get().is_nan() || y2.get().is_nan())
    });

    view! {
        <g
            class=format!("_chartistry_{}_guide_line", id)
            stroke=move || color.get().to_string()
            stroke-width=width>
            <Show when=move || hover_inner.get() && have_data.get() >
                <DebugRect label=format!("{}_guide_line", id) debug=debug />
                <line
                    x1=x1
                    y1=y1
                    x2=x2
                    y2=y2
                />
            </Show>
        </g>
    }
}
