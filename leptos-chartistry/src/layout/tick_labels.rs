use super::{UseLayout, UseVerticalLayout};
use crate::{
    bounds::Bounds,
    debug::DebugRect,
    edge::Edge,
    series::Range,
    state::{PreState, State},
    ticks::{
        AlignedFloats, GeneratedTicks, HorizontalSpan, TickFormat, TickFormatFn, TickGen,
        VerticalSpan,
    },
    Tick,
};
#[cfg(feature = "timestamps")]
use crate::ticks::Timestamps;
use leptos::prelude::*;
use std::sync::Arc;

/// Builds tick labels for an axis.
///
/// Note that ticks lack an identity resulting in generators and labels not being reactive.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct TickLabels<XY: Tick> {
    /// Minimum number of characters to display for each tick label.
    ///
    /// Helpful for giving a fixed width to labels e.g., if your graph can display 0-100 then it might show a shorter label on "0" or "42" to "100". Needed to have the same inner chart ratio when using an outer chart ratio. Can also be useful for aligning a list of charts.
    pub min_chars: RwSignal<usize>,
    /// Format function for the tick labels. See [TickLabels::with_format] for details.
    pub format: RwSignal<Arc<TickFormatFn<XY>>>,
    /// Tick generator for the labels.
    pub generator: RwSignal<Arc<dyn TickGen<Tick = XY> + Send + Sync>>,
}

/// Whether a tick label represents X or Y data, used for projection positioning.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum DataAxis {
    X,
    Y,
}

#[derive(Clone)]
pub struct UseTickLabels {
    ticks: Signal<Vec<(f64, String)>>,
    data_axis: DataAxis,
}

impl<XY: Tick> Clone for TickLabels<XY> {
    fn clone(&self) -> Self {
        Self {
            min_chars: self.min_chars,
            format: self.format,
            generator: self.generator,
        }
    }
}

impl<XY: Tick> Default for TickLabels<XY> {
    fn default() -> Self {
        Self::from_generator(XY::tick_label_generator())
    }
}

impl TickLabels<f64> {
    /// Creates a new tick label generator for floating point numbers. See [AlignedFloats] for details.
    pub fn aligned_floats() -> Self {
        Self::from_generator(AlignedFloats::default())
    }
}

#[cfg(feature = "timestamps")]
impl<Tz> TickLabels<chrono::DateTime<Tz>>
where
    Tz: chrono::TimeZone + Send + Sync + 'static,
    Tz::Offset: std::fmt::Display + Send + Sync,
{
    /// Creates a new tick label generator for timestamps. See [Timestamps] for details.
    pub fn timestamps() -> Self {
        Self::from_generator(Timestamps::default())
    }
}

impl<XY: Tick> TickLabels<XY> {
    /// Creates a new tick label generator from a tick generator. Use [AlignedFloats] or [Timestamps] for available generators.
    pub fn from_generator(gen: impl TickGen<Tick = XY> + 'static) -> Self {
        Self {
            min_chars: RwSignal::default(),
            format: RwSignal::new(HorizontalSpan::identity_format()),
            generator: RwSignal::new(Arc::new(gen)),
        }
    }

    /// Sets the minimum number of characters to display for each tick label.
    pub fn with_min_chars(self, min_chars: usize) -> Self {
        self.min_chars.set(min_chars);
        self
    }

    /// Sets the format function for the tick labels.
    ///
    /// This is a function that takes a `Tick` and a formatter and returns a `String`. It gives an opportunity to customise tick label format. The formatter is the resulting state of the tick generator and does the default aciton. For example if aligned floats decides to use "1000s" then the formatter will use that.
    pub fn with_format(
        self,
        format: impl Fn(&XY, &dyn TickFormat<Tick = XY>) -> String + Send + Sync + 'static,
    ) -> Self {
        self.format.set(Arc::new(format));
        self
    }

    fn map_ticks(&self, gen: Memo<GeneratedTicks<XY>>) -> Signal<Vec<(f64, String)>> {
        let format = self.format;
        Signal::derive(move || {
            let format = format.get();
            gen.with(|GeneratedTicks { ticks, state }| {
                ticks
                    .iter()
                    .map(|tick| (tick.position(), (format)(tick, state.as_ref())))
                    .collect()
            })
        })
    }
}

impl<Gen, XY> From<Gen> for TickLabels<XY>
where
    Gen: TickGen<Tick = XY> + 'static,
    XY: Tick,
{
    fn from(gen: Gen) -> Self {
        Self::from_generator(gen)
    }
}

impl<XY: Tick> TickLabels<XY> {
    /// Generate ticks for display on a horizontal physical edge.
    pub(crate) fn generate_horizontal<X: Tick, Y: Tick>(
        &self,
        range: Memo<Range<XY>>,
        state: &PreState<X, Y>,
        avail_width: Signal<f64>,
    ) -> Memo<GeneratedTicks<XY>> {
        let font_width = state.font_width;
        let padding = state.padding;
        let TickLabels {
            min_chars,
            format,
            generator,
        } = self.clone();
        Memo::new(move |_| {
            range
                .get()
                .range()
                .map(|(first, last)| {
                    let span = HorizontalSpan::new(
                        font_width.get(),
                        min_chars.get(),
                        padding.get().width(),
                        avail_width.get(),
                        format.get(),
                    );
                    generator.get().generate(first, last, &span)
                })
                .unwrap_or_else(GeneratedTicks::none)
        })
    }

    /// Generate ticks for display on a vertical physical edge.
    pub(crate) fn generate_vertical<X: Tick, Y: Tick>(
        &self,
        range: Memo<Range<XY>>,
        state: &PreState<X, Y>,
        avail_height: Signal<f64>,
    ) -> Memo<GeneratedTicks<XY>> {
        let font_height = state.font_height;
        let padding = state.padding;
        let generator = self.generator;
        Memo::new(move |_| {
            range
                .get()
                .range()
                .map(|(first, last)| {
                    let span = VerticalSpan::new(
                        font_height.get() + padding.get().height(),
                        avail_height.get(),
                    );
                    generator.get().generate(first, last, &span)
                })
                .unwrap_or_else(GeneratedTicks::none)
        })
    }

    pub(super) fn fixed_height<X: Tick, Y: Tick>(&self, state: &PreState<X, Y>) -> Signal<f64> {
        let font_height = state.font_height;
        let padding = state.padding;
        Signal::derive(move || font_height.get() + padding.get().height())
    }

    pub(super) fn to_horizontal_use<X: Tick, Y: Tick>(
        &self,
        range: Memo<Range<XY>>,
        state: &PreState<X, Y>,
        avail_width: Memo<f64>,
        data_axis: DataAxis,
    ) -> UseLayout {
        UseLayout::TickLabels(UseTickLabels {
            ticks: self.map_ticks(self.generate_horizontal(range, state, avail_width.into())),
            data_axis,
        })
    }

    pub(super) fn to_vertical_use<X: Tick, Y: Tick>(
        &self,
        range: Memo<Range<XY>>,
        state: &PreState<X, Y>,
        avail_height: Memo<f64>,
        data_axis: DataAxis,
    ) -> UseVerticalLayout {
        let ticks = self.map_ticks(self.generate_vertical(range, state, avail_height.into()));
        UseVerticalLayout {
            width: mk_width(self.min_chars, state, ticks),
            layout: UseLayout::TickLabels(UseTickLabels {
                ticks,
                data_axis,
            }),
        }
    }
}

fn mk_width<X: Tick, Y: Tick>(
    min_chars: RwSignal<usize>,
    state: &PreState<X, Y>,
    ticks: Signal<Vec<(f64, String)>>,
) -> Signal<f64> {
    let font_width = state.font_width;
    let padding = state.padding;
    Signal::derive(move || {
        let longest_chars = ticks.with(|ticks| {
            ticks
                .iter()
                .map(|(_, label)| label.len())
                .max()
                .unwrap_or_default()
                .max(min_chars.get())
        }) as f64;
        font_width.get() * longest_chars + padding.get().width()
    })
}

fn align_tick_labels(labels: Vec<String>) -> Vec<String> {
    // Find longest label length
    let min_label = labels
        .iter()
        .map(|label| label.len())
        .max()
        .unwrap_or_default();
    // Pad labels to same length
    labels
        .into_iter()
        .map(|mut label| {
            let spaces = " ".repeat(min_label.saturating_sub(label.len()));
            label.insert_str(0, &spaces);
            label
        })
        .collect::<Vec<_>>()
}

#[component]
pub(super) fn TickLabels<X: Tick, Y: Tick>(
    ticks: UseTickLabels,
    edge: Edge,
    bounds: Memo<Bounds>,
    state: State<X, Y>,
) -> impl IntoView {
    let data_axis = ticks.data_axis;
    let ticks = move || {
        // Align vertical labels
        let ticks = ticks.ticks.get();
        let ticks = if edge.is_vertical() {
            let (pos, labels): (Vec<f64>, Vec<String>) = ticks.into_iter().unzip();
            let labels = align_tick_labels(labels);
            pos.into_iter().zip(labels).collect::<Vec<_>>()
        } else {
            ticks
        };
        ticks
            .into_iter()
            .map(|tick| {
                view! {
                    <TickLabel edge=edge data_axis=data_axis outer=bounds state=state.clone() tick=tick />
                }
            })
            .collect_view()
    };
    view! {
        <g class="_chartistry_tick_labels">
            {ticks}
        </g>
    }
}

#[component]
fn TickLabel<X: Tick, Y: Tick>(
    edge: Edge,
    data_axis: DataAxis,
    outer: Memo<Bounds>,
    state: State<X, Y>,
    tick: (f64, String),
) -> impl IntoView {
    let debug = state.pre.debug;
    let font_height = state.pre.font_height;
    let font_width = state.pre.font_width;
    let padding = state.pre.padding;

    // Select projection: Y data uses Primary/Secondary based on edge side,
    // X data always uses primary
    let projection = match data_axis {
        DataAxis::X => state.projection_primary,
        DataAxis::Y => {
            // For Y data, "start" side uses primary, "end" side uses secondary.
            // In normal mode: left=primary, right=secondary
            // In rotated mode: top=primary, bottom=secondary
            // The edge assignment already handles this mapping in compose()
            match edge {
                Edge::Right | Edge::Bottom => state.projection_secondary,
                _ => state.projection_primary,
            }
        }
    };

    let (position, label) = tick;
    let label_len = label.len();
    // Calculate positioning Bounds. Note: tick w / h includes padding
    let bounds = Signal::derive(move || {
        let padding = padding.get();
        let width = font_width.get() * label_len as f64 + padding.width();
        let height = font_height.get() + padding.height();

        let proj = projection.get();
        let outer = outer.get();

        // Get SVG position based on data axis
        let svg_pos = match data_axis {
            DataAxis::X => proj.position_to_svg(position, 0.0),
            DataAxis::Y => proj.position_to_svg(0.0, position),
        };

        // Position based on physical edge
        match edge {
            Edge::Top | Edge::Bottom => {
                let x = svg_pos.0 - width / 2.0;
                Bounds::from_points(x, outer.top_y(), x + width, outer.bottom_y())
            }

            Edge::Left | Edge::Right => {
                let y = svg_pos.1 - height / 2.0;
                Bounds::from_points(outer.left_x(), y, outer.right_x(), y + height)
            }
        }
    });
    let content = Memo::new(move |_| padding.get().apply(bounds.get()));

    // Determine text position
    let text_position = Memo::new(move |_| {
        let content = content.get();
        match edge {
            Edge::Top | Edge::Bottom => ("middle", content.centre_x()),

            Edge::Left | Edge::Right => {
                let (x, anchor) = if edge == Edge::Left {
                    (content.right_x(), "end")
                } else {
                    (content.left_x(), "start")
                };
                (anchor, x)
            }
        }
    });

    view! {
        <g
            class="_chartistry_tick_label"
            font-family="monospace">
            <DebugRect label="tick" debug=debug bounds=vec![bounds, content.into()] />
            <text
                x=move || text_position.get().1
                y=move || content.get().centre_y()
                style="white-space: pre;"
                font-size=move || font_height.get()
                dominant-baseline="middle"
                text-anchor=move || text_position.get().0>
                {label.clone()}
            </text>
        </g>
    }
}
