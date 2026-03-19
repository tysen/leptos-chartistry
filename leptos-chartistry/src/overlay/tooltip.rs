use crate::{
    debug::DebugRect,
    series::{Snippet, UseY, YAxis},
    state::State,
    Tick, TickLabels, AXIS_MARKER_COLOUR,
};
use leptos::prelude::*;
use std::{
    cmp::{Ordering, Reverse},
    sync::Arc,
};

/// Default gap distance from cursor to tooltip when shown.
pub const TOOLTIP_CURSOR_DISTANCE: f64 = 10.0;

/// Data available to custom tooltip renderers.
#[derive(Clone)]
#[non_exhaustive]
pub struct TooltipData<X: Tick, Y: Tick> {
    /// The nearest X data value under the cursor.
    pub nearest_x: Memo<Option<X>>,
    /// Y values per series at nearest X, sorted and filtered per tooltip config.
    pub nearest_y: Memo<Vec<(UseY, Option<Y>)>>,
    /// Full chart state (mouse position, projections, layout, etc.).
    pub state: State<X, Y>,
}

/// Custom tooltip body renderer.
pub type TooltipBodyFn<X, Y> = Arc<dyn Fn(TooltipData<X, Y>) -> AnyView + Send + Sync>;
/// Custom tooltip X header renderer.
pub type TooltipXHeaderFn<X> = Arc<dyn Fn(Memo<Option<X>>) -> AnyView + Send + Sync>;
/// Custom tooltip Y row renderer.
pub type TooltipYRowFn<Y> = Arc<dyn Fn(UseY, Option<Y>, String) -> AnyView + Send + Sync>;

/// Builds a mouse tooltip that shows X and Y values for the nearest data. Drawn in HTML as an overlay.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Tooltip<X: Tick, Y: Tick> {
    /// Where the tooltip is placed when shown.
    pub placement: RwSignal<TooltipPlacement>,
    /// How the tooltip Y value table is sorted.
    pub sort_by: RwSignal<TooltipSortBy>,
    /// Gap distance from cursor to tooltip when shown.
    pub cursor_distance: RwSignal<f64>,
    /// If true, skips Y values that are `f64::NAN`.
    pub skip_missing: RwSignal<bool>,
    /// Whether to show X ticks. Default is true.
    // TODO: move to TickLabels
    pub show_x_ticks: RwSignal<bool>,
    /// Whether to show Y series snippets (colored line indicators). Default is true.
    pub show_y_snippets: RwSignal<bool>,
    /// X axis formatter.
    pub x_ticks: TickLabels<X>,
    /// Y axis formatter for the primary (left) axis.
    pub y_ticks: TickLabels<Y>,
    /// Y axis formatter for the secondary (right) axis.
    /// If None, falls back to using y_ticks for all series.
    pub y_ticks_secondary: Option<TickLabels<Y>>,

    /// Custom view for the entire tooltip body. Replaces the default header + table.
    /// When set, `x_header` and `y_row` are ignored.
    pub body: Option<RwSignal<TooltipBodyFn<X, Y>>>,
    /// Custom view for the X header. Replaces the default `<h2>` with formatted text.
    /// Ignored if `body` is set.
    pub x_header: Option<RwSignal<TooltipXHeaderFn<X>>>,
    /// Custom view for each Y series row. Replaces the default `<tr>` with snippet + text value.
    /// Receives: series info, raw Y value, and the pre-formatted string.
    /// Ignored if `body` is set.
    pub y_row: Option<RwSignal<TooltipYRowFn<Y>>>,
}

/// Where the tooltip is place when shown.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum TooltipPlacement {
    /// Does not show a tooltip.
    #[default]
    Hide,
    /// Shows the tooltip to the left of the cursor.
    LeftCursor,
}

/// How the tooltip Y value table is sorted.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum TooltipSortBy {
    /// Sorts by line name.
    #[default]
    Lines,
    /// Sorts by Y value in ascending order.
    Ascending,
    /// Sorts by Y value in descending order.
    Descending,
}

impl<X: Tick, Y: Tick> Tooltip<X, Y> {
    /// Creates a new tooltip with the given placement, X ticks, and Y ticks.
    pub fn new(
        placement: impl Into<TooltipPlacement>,
        x_ticks: impl Into<TickLabels<X>>,
        y_ticks: impl Into<TickLabels<Y>>,
    ) -> Self {
        Self {
            placement: RwSignal::new(placement.into()),
            x_ticks: x_ticks.into(),
            y_ticks: y_ticks.into(),
            y_ticks_secondary: None,
            ..Default::default()
        }
    }

    /// Creates a new tooltip with the given placement. Uses default X and Y ticks.
    pub fn from_placement(placement: impl Into<TooltipPlacement>) -> Self {
        Self::new(
            placement,
            TickLabels::from_generator(X::tooltip_generator()),
            TickLabels::from_generator(Y::tooltip_generator()),
        )
    }

    /// Creates a new tooltip left of the cursor. Uses default X and Y ticks.
    pub fn left_cursor() -> Self {
        Self::from_placement(TooltipPlacement::LeftCursor)
    }

    /// Sets the sort order of the Y value table.
    pub fn with_sort_by(self, sort_by: impl Into<TooltipSortBy>) -> Self {
        self.sort_by.set(sort_by.into());
        self
    }

    /// Sets the gap distance from cursor to tooltip when shown.
    pub fn with_cursor_distance(self, distance: impl Into<f64>) -> Self {
        self.cursor_distance.set(distance.into());
        self
    }

    /// Sets whether the tooltip should skip Y values that are `f64::NAN`.
    pub fn skip_missing(self, skip_missing: impl Into<bool>) -> Self {
        self.skip_missing.set(skip_missing.into());
        self
    }

    /// Sets whether to show X ticks.
    pub fn show_x_ticks(self, show_x_ticks: impl Into<bool>) -> Self {
        self.show_x_ticks.set(show_x_ticks.into());
        self
    }

    /// Sets whether to show Y series snippets (colored line indicators).
    pub fn show_y_snippets(self, show_y_snippets: impl Into<bool>) -> Self {
        self.show_y_snippets.set(show_y_snippets.into());
        self
    }

    /// Sets the Y tick formatter for the secondary (right) axis.
    pub fn with_y_ticks_secondary(mut self, y_ticks: impl Into<TickLabels<Y>>) -> Self {
        self.y_ticks_secondary = Some(y_ticks.into());
        self
    }

    /// Sets a custom view for the entire tooltip body. Replaces the default header + table.
    /// When set, `x_header` and `y_row` are ignored.
    pub fn with_body(
        mut self,
        f: impl Fn(TooltipData<X, Y>) -> AnyView + Send + Sync + 'static,
    ) -> Self {
        self.body = Some(RwSignal::new(Arc::new(f)));
        self
    }

    /// Sets a custom view for the X header. Replaces the default `<h2>` with formatted text.
    /// Ignored if `body` is set.
    pub fn with_x_header(
        mut self,
        f: impl Fn(Memo<Option<X>>) -> AnyView + Send + Sync + 'static,
    ) -> Self {
        self.x_header = Some(RwSignal::new(Arc::new(f)));
        self
    }

    /// Sets a custom view for each Y series row. Replaces the default `<tr>` with snippet + text value.
    /// Receives: series info, raw Y value, and the pre-formatted string.
    /// Ignored if `body` is set.
    pub fn with_y_row(
        mut self,
        f: impl Fn(UseY, Option<Y>, String) -> AnyView + Send + Sync + 'static,
    ) -> Self {
        self.y_row = Some(RwSignal::new(Arc::new(f)));
        self
    }
}

impl<X: Tick, Y: Tick> Default for Tooltip<X, Y> {
    fn default() -> Self {
        Self {
            placement: RwSignal::default(),
            sort_by: RwSignal::default(),
            cursor_distance: RwSignal::new(TOOLTIP_CURSOR_DISTANCE),
            skip_missing: RwSignal::new(false),
            show_x_ticks: RwSignal::new(true),
            show_y_snippets: RwSignal::new(true),
            x_ticks: TickLabels::default(),
            y_ticks: TickLabels::default(),
            y_ticks_secondary: None,
            body: None,
            x_header: None,
            y_row: None,
        }
    }
}

impl TooltipSortBy {
    fn to_ord<Y: Tick>(y: &Option<Y>) -> Option<F64Ord> {
        y.as_ref().map(|y| F64Ord(y.position()))
    }

    fn sort_values<Y: Tick>(&self, values: &mut [(UseY, Option<Y>)]) {
        match self {
            TooltipSortBy::Lines => values.sort_by_key(|(line, _)| line.name.get()),
            TooltipSortBy::Ascending => values.sort_by_key(|(_, y)| Self::to_ord(y)),
            TooltipSortBy::Descending => values.sort_by_key(|(_, y)| Reverse(Self::to_ord(y))),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct F64Ord(f64);

impl PartialOrd for F64Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F64Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Eq for F64Ord {}

impl std::fmt::Display for TooltipPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TooltipPlacement::Hide => write!(f, "Hide"),
            TooltipPlacement::LeftCursor => write!(f, "Left cursor"),
        }
    }
}

impl std::str::FromStr for TooltipPlacement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hide" => Ok(TooltipPlacement::Hide),
            "left cursor" => Ok(TooltipPlacement::LeftCursor),
            _ => Err(format!("invalid TooltipPlacement: `{}`", s)),
        }
    }
}

impl std::fmt::Display for TooltipSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TooltipSortBy::Lines => write!(f, "Lines"),
            TooltipSortBy::Ascending => write!(f, "Ascending"),
            TooltipSortBy::Descending => write!(f, "Descending"),
        }
    }
}

impl std::str::FromStr for TooltipSortBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lines" => Ok(TooltipSortBy::Lines),
            "ascending" => Ok(TooltipSortBy::Ascending),
            "descending" => Ok(TooltipSortBy::Descending),
            _ => Err(format!("invalid SortBy: `{}`", s)),
        }
    }
}

#[component]
pub(crate) fn Tooltip<X: Tick, Y: Tick>(
    tooltip: Tooltip<X, Y>,
    state: State<X, Y>,
) -> impl IntoView {
    let Tooltip {
        placement,
        sort_by,
        skip_missing,
        cursor_distance,
        show_x_ticks,
        show_y_snippets,
        x_ticks,
        y_ticks,
        y_ticks_secondary,
        body,
        x_header,
        y_row,
    } = tooltip;
    let debug = state.pre.debug;
    let font_height = state.pre.font_height;
    let font_width = state.pre.font_width;
    let padding = state.pre.padding;
    let inner = state.layout.inner;

    // Nearest X data value
    let nearest_data_x = state.pre.data.nearest_data_x(state.hover_position_x);

    let x_body = {
        let x_format = x_ticks.format;
        let avail_width = Signal::derive(move || inner.read().width());
        let x_ticks = x_ticks.generate_horizontal(state.pre.data.range_x, &state.pre, avail_width);
        Signal::derive(move || {
            // Hide ticks?
            if !show_x_ticks.get() {
                return "".to_string();
            }
            let x_format = x_format.get();
            nearest_data_x.read().as_ref().map_or_else(
                || "no data".to_string(),
                |x_value| (x_format)(x_value, x_ticks.read().state.as_ref()),
            )
        })
    };

    // Primary axis Y formatter
    let avail_height = Signal::derive(move || inner.read().height());
    let y_format_primary = y_ticks.format;
    let y_ticks_primary = y_ticks.generate_vertical(state.pre.data.range_y_primary, &state.pre, avail_height);

    // Secondary axis Y formatter (falls back to primary if not set)
    let (y_format_secondary, y_ticks_secondary_gen) = if let Some(y_ticks_sec) = y_ticks_secondary {
        let format = y_ticks_sec.format;
        let ticks = y_ticks_sec.generate_vertical(state.pre.data.range_y_secondary, &state.pre, avail_height);
        (format, ticks)
    } else {
        // Fall back to primary formatter for secondary axis too
        (y_format_primary, y_ticks_primary)
    };

    let format_y_value: Arc<dyn Fn(YAxis, Option<Y>) -> String + Send + Sync> =
        Arc::new(move |axis: YAxis, y_value: Option<Y>| {
            let (y_format, y_ticks) = match axis {
                YAxis::Primary => (y_format_primary, y_ticks_primary),
                YAxis::Secondary => (y_format_secondary, y_ticks_secondary_gen),
            };
            let y_format = y_format.get();
            y_value.as_ref().map_or_else(
                || "-".to_string(),
                |y_value| (y_format)(y_value, y_ticks.read().state.as_ref()),
            )
        });

    let nearest_y_values = {
        let nearest_data_y = state.pre.data.nearest_data_y(state.hover_position_x);
        Memo::new(move |_| {
            let mut y_values = nearest_data_y.get();
            // Skip missing?
            if skip_missing.get() {
                y_values = y_values
                    .into_iter()
                    .filter(|(_, y_value)| y_value.is_some())
                    .collect::<Vec<_>>()
            }
            // Sort values
            sort_by.get().sort_values(&mut y_values);
            y_values
        })
    };

    // Build tooltip data for custom renderers
    let tooltip_data = TooltipData {
        nearest_x: nearest_data_x,
        nearest_y: nearest_y_values,
        state: state.clone(),
    };

    // Store non-Copy values for use inside <Show> children (needs Fn, not FnOnce)
    let format_y_value = StoredValue::new(format_y_value);
    let tooltip_data = StoredValue::new(tooltip_data);
    let state_stored = StoredValue::new(state.clone());

    view! {
        <Show when=move || state.hover_inner.get() && placement.get() != TooltipPlacement::Hide>
            <DebugRect label="tooltip" debug=debug />
            <aside
                class="_chartistry_tooltip"
                style="position: absolute; z-index: 1; width: max-content; height: max-content; transform: translateY(-50%); background-color: #fff; white-space: pre; font-family: monospace;"
                style:border=format!("1px solid {}", AXIS_MARKER_COLOUR)
                style:top=move || format!("calc({}px)", state.mouse_page.get().1)
                style:right=move || format!("calc(100% - {}px + {}px)", state.mouse_page.get().0, cursor_distance.get())
                style:padding=move || padding.get().to_css_style()>
                {move || {
                    if let Some(body) = body {
                        (body.get())(tooltip_data.get_value())
                    } else {
                        // X header: custom or default
                        let x_header_view: AnyView = if let Some(x_header_fn) = x_header {
                            (x_header_fn.get())(nearest_data_x)
                        } else {
                            view! {
                                <h2
                                    style="margin: 0; text-align: center;"
                                    style:font-size=move || format!("{}px", font_height.get())>
                                    {move || x_body.get()}
                                </h2>
                            }
                            .into_any()
                        };

                        // Y rows with formatted values
                        let fmt = format_y_value.get_value();
                        let nearest_data_y = move || {
                            nearest_y_values
                                .get()
                                .into_iter()
                                .map(|(line, y_value)| {
                                    let formatted = fmt(line.axis, y_value.clone());
                                    (line, y_value, formatted)
                                })
                                .collect::<Vec<_>>()
                        };

                        let state_for_rows = state_stored.get_value();
                        let series_tr = move |(series, y_value, formatted): (UseY, Option<Y>, String)| {
                            if let Some(ref y_row_fn) = y_row {
                                (y_row_fn.get())(series, y_value, formatted)
                            } else {
                                let state = state_for_rows.clone();
                                let text_align = move || if show_y_snippets.get() { "right" } else { "center" };
                                view! {
                                    <tr>
                                        <Show when=move || show_y_snippets.get()>
                                            <td><Snippet series=series.clone() state=state.clone() /></td>
                                        </Show>
                                        <td
                                            style="white-space: pre; font-family: monospace;"
                                            style:text-align=text_align
                                            style:padding-top=move || format!("{}px", font_height.get() / 4.0)
                                            style:padding-left=move || format!("{}px", font_width.get())>
                                            {formatted}
                                        </td>
                                    </tr>
                                }
                                .into_any()
                            }
                        };

                        view! {
                            {x_header_view}
                            <table
                                style="border-collapse: collapse; border-spacing: 0; padding: 0;"
                                style:margin=move || if show_y_snippets.get() { "0 0 0 auto" } else { "0 auto" }
                                style:font-size=move || format!("{}px", font_height.get())>
                                <tbody>
                                    <For
                                        each=nearest_data_y
                                        key=|(series, _, formatted)| (series.id, formatted.to_owned())
                                        children=series_tr
                                    />
                                </tbody>
                            </table>
                        }
                        .into_any()
                    }
                }}
            </aside>
        </Show>
    }.into_any()
}
