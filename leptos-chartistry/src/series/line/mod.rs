mod interpolation;
mod marker;
pub use interpolation::{Interpolation, Step};
pub use marker::{Marker, MarkerShape};

use super::{ApplyUseSeries, IntoUseLine, SeriesAcc, UseData, UseY, YAxis};
use crate::{
    bounds::Bounds,
    colors::{Color, DivergingGradient, LinearGradientSvg, SequentialGradient, BERLIN, LIPARI},
    series::GetYValue,
    ColorScheme, Tick,
};
use leptos::prelude::*;
use std::sync::Arc;

/// Suggested color scheme for a linear gradient on a line. Uses darker colors for lower values and lighter colors for higher values. Assumes a light background.
pub const LINEAR_GRADIENT: SequentialGradient = LIPARI;

/// Suggested color scheme for a diverging gradient on a line. Uses a blue for negative values, a dark central value and red for positive values. Assumes a light background.
pub const DIVERGING_GRADIENT: DivergingGradient = BERLIN;

/// Draws a line on the chart.
///
/// # Simple example
/// With no legend names, lines can be a simple closure:
/// ```rust
/// # use leptos_chartistry::*;
/// # struct MyData { x: f64, y1: f64, y2: f64 }
/// let series = Series::new(|data: &MyData| data.x)
///     .line(|data: &MyData| data.y1)
///     .line(|data: &MyData| data.y2);
/// ```
/// See this in action with the [tick labels example](https://feral-dot-io.github.io/leptos-chartistry/examples.html#tick-labels).
///
/// # Example
/// However, we can also set the name of the line which a legend can show:
/// ```rust
/// # use leptos_chartistry::*;
/// # struct MyData { x: f64, y1: f64, y2: f64 }
/// let series = Series::new(|data: &MyData| data.x)
///     .line(Line::new(|data: &MyData| data.y1).with_name("pears"))
///     .line(Line::new(|data: &MyData| data.y2).with_name("apples"));
/// ```
/// See this in action with the [legend example](https://feral-dot-io.github.io/leptos-chartistry/examples.html#legend).
///
/// # Dual Y-axis example
/// Lines can be assigned to either the primary (left) or secondary (right) Y-axis:
/// ```rust
/// # use leptos_chartistry::*;
/// # struct MyData { x: f64, y1: f64, y2: f64 }
/// let series = Series::new(|data: &MyData| data.x)
///     .line(Line::new(|data: &MyData| data.y1))  // Primary (left) axis
///     .line(Line::new(|data: &MyData| data.y2)
///         .with_y_axis(YAxis::Secondary));       // Secondary (right) axis
/// ```
#[non_exhaustive]
pub struct Line<T, Y> {
    get_y: Arc<dyn GetYValue<T, Y>>,
    /// Name of the line. Used in the legend.
    pub name: RwSignal<String>,
    /// Which Y-axis to plot this line against. Default is [YAxis::Primary] (left).
    pub axis: RwSignal<YAxis>,
    /// Color of the line. If not set, the next color in the series will be used.
    pub color: RwSignal<Option<Color>>,
    /// Use a linear gradient (color scheme) for the line. Default is `None` with fallback to the line color.
    pub gradient: RwSignal<Option<ColorScheme>>,
    /// Width of the line.
    pub width: RwSignal<f64>,
    /// Interpolation method of the line, aka line smoothing (or not). Describes how the line is drawn between two points. Default is [Interpolation::Monotone].
    pub interpolation: RwSignal<Interpolation>,
    /// Marker at each point on the line.
    pub marker: Marker,
    /// Fill color for the region above the line (higher Y values).
    pub fill_above: RwSignal<Option<Color>>,
    /// Fill color for the region below the line (lower Y values).
    pub fill_below: RwSignal<Option<Color>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UseLine {
    color: Signal<Color>,
    gradient: RwSignal<Option<ColorScheme>>,
    width: RwSignal<f64>,
    interpolation: RwSignal<Interpolation>,
    marker: Marker,
    fill_above: RwSignal<Option<Color>>,
    fill_below: RwSignal<Option<Color>>,
}

impl<T, Y> Line<T, Y> {
    /// Create a new line. The `get_y` function is used to extract the Y value from your struct.
    ///
    /// See the module documentation for examples.
    pub fn new(get_y: impl Fn(&T) -> Y + Send + Sync + 'static) -> Self
    where
        Y: Tick,
    {
        Self {
            get_y: Arc::new(get_y),
            name: RwSignal::default(),
            axis: RwSignal::default(),
            color: RwSignal::default(),
            gradient: RwSignal::default(),
            width: RwSignal::new(1.0),
            interpolation: RwSignal::default(),
            marker: Marker::default(),
            fill_above: RwSignal::default(),
            fill_below: RwSignal::default(),
        }
    }

    /// Set the name of the line. Used in the legend.
    pub fn with_name(self, name: impl Into<String>) -> Self {
        self.name.set(name.into());
        self
    }

    /// Set which Y-axis to plot this line against.
    ///
    /// Default is [YAxis::Primary] (left). Use [YAxis::Secondary] for the right axis.
    pub fn with_y_axis(self, axis: YAxis) -> Self {
        self.axis.set(axis);
        self
    }

    /// Set the color of the line. If not set, the next color in the series will be used.
    pub fn with_color(self, color: impl Into<Option<Color>>) -> Self {
        self.color.set(color.into());
        self
    }

    /// Use a color scheme for the line. Interpolated in SVG by the browser, overrides [Color]. Default is `None` with fallback to the line color.
    ///
    /// Suggested use with [LINEAR_GRADIENT] or [DIVERGING_GRADIENT] (for data with a zero value).
    pub fn with_gradient(self, scheme: impl Into<ColorScheme>) -> Self {
        self.gradient.set(Some(scheme.into()));
        self
    }

    /// Set the width of the line.
    pub fn with_width(self, width: impl Into<f64>) -> Self {
        self.width.set(width.into());
        self
    }

    /// Set the interpolation method of the line.
    pub fn with_interpolation(self, interpolation: impl Into<Interpolation>) -> Self {
        self.interpolation.set(interpolation.into());
        self
    }

    /// Set the marker at each point on the line.
    pub fn with_marker(mut self, marker: impl Into<Marker>) -> Self {
        self.marker = marker.into();
        self
    }

    /// Set the fill color for the region above the line (higher Y values).
    pub fn with_fill_above(self, color: impl Into<Color>) -> Self {
        self.fill_above.set(Some(color.into()));
        self
    }

    /// Set the fill color for the region below the line (lower Y values).
    pub fn with_fill_below(self, color: impl Into<Color>) -> Self {
        self.fill_below.set(Some(color.into()));
        self
    }
}

impl<T, Y> Clone for Line<T, Y> {
    fn clone(&self) -> Self {
        Self {
            get_y: self.get_y.clone(),
            name: self.name,
            axis: self.axis,
            color: self.color,
            gradient: self.gradient,
            width: self.width,
            interpolation: self.interpolation,
            marker: self.marker.clone(),
            fill_above: self.fill_above,
            fill_below: self.fill_below,
        }
    }
}

impl<T, Y: Tick, F: Fn(&T) -> Y + Send + Sync + 'static> From<F> for Line<T, Y> {
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

impl<T, Y: Tick, U: Fn(&T) -> Y + Send + Sync> GetYValue<T, Y> for U {
    fn value(&self, t: &T) -> Y {
        self(t)
    }

    fn stacked_value(&self, t: &T) -> Y {
        self(t)
    }
}

impl<T, Y> ApplyUseSeries<T, Y> for Line<T, Y> {
    fn apply_use_series(self: Arc<Self>, series: &mut SeriesAcc<T, Y>) {
        let color = series.next_color();
        // Read axis value during setup - changing axis dynamically requires rebuilding the series
        let axis = self.axis.get_untracked();
        _ = series.push_line(color, axis, (*self).clone());
    }
}

impl<T, Y> IntoUseLine<T, Y> for Line<T, Y> {
    fn into_use_line(
        self,
        id: usize,
        color: Memo<Color>,
        axis: YAxis,
    ) -> (UseY, Arc<dyn GetYValue<T, Y>>) {
        let override_color = self.color;
        let color = Signal::derive(move || override_color.get().unwrap_or(color.get()));
        let line = UseY::new_line(
            id,
            self.name,
            axis,
            UseLine {
                color,
                gradient: self.gradient,
                width: self.width,
                interpolation: self.interpolation,
                marker: self.marker.clone(),
                fill_above: self.fill_above,
                fill_below: self.fill_below,
            },
        );
        (line, self.get_y.clone())
    }
}

#[component]
pub fn RenderLine<X: Tick, Y: Tick>(
    use_y: UseY,
    line: UseLine,
    data: UseData<X, Y>,
    positions: Signal<Vec<(f64, f64)>>,
    markers: Signal<Vec<(f64, f64)>>,
    x_is_horizontal: bool,
    #[prop(optional)] inner_bounds: Option<Memo<Bounds>>,
) -> impl IntoView {
    let path = move || {
        positions.with(|positions| line.interpolation.get().path(positions, x_is_horizontal))
    };

    // Line color
    let gradient_id = format!("line_{}_gradient", use_y.id);
    let stroke = {
        let color = line.color;
        let gradient_id = gradient_id.clone();
        Signal::derive(move || {
            // Gradient takes precedence
            if line.gradient.get().is_some() {
                format!("url(#{gradient_id})")
            } else {
                color.get().to_string()
            }
        })
    };
    let gradient = Signal::derive(move || {
        line.gradient
            .get()
            .unwrap_or_else(|| LINEAR_GRADIENT.into())
    });
    // Select the appropriate Y range based on which axis this line uses
    let axis = use_y.axis;
    let range_y = Signal::derive(move || {
        let range = match axis {
            YAxis::Primary => data.range_y_primary,
            YAxis::Secondary => data.range_y_secondary,
        };
        range.read().positions()
    });

    // Area fill via clipPath
    let clip_above_id = format!("line_{}_clip_above", use_y.id);
    let clip_below_id = format!("line_{}_clip_below", use_y.id);
    let clip_above_url = format!("url(#{})", clip_above_id);
    let clip_below_url = format!("url(#{})", clip_below_id);

    // Build clip polygon paths from the line path + chart edge corners
    let clip_above_path = move || {
        let bounds = inner_bounds.map(|b| b.get());
        let Some(bounds) = bounds else { return String::new(); };
        positions.with(|positions| {
            build_clip_path(positions, &bounds, x_is_horizontal, true, line.interpolation.get())
        })
    };
    let clip_below_path = move || {
        let bounds = inner_bounds.map(|b| b.get());
        let Some(bounds) = bounds else { return String::new(); };
        positions.with(|positions| {
            build_clip_path(positions, &bounds, x_is_horizontal, false, line.interpolation.get())
        })
    };

    let fill_above = line.fill_above;
    let fill_below = line.fill_below;

    let width = line.width;
    view! {
        <g
            class="_chartistry_line"
            stroke=stroke
            stroke-linecap="round"
            stroke-linejoin="bevel"
            stroke-width=width>
            <defs>
                <Show when=move || line.gradient.get().is_some()>
                    <LinearGradientSvg
                        id=gradient_id.clone()
                        scheme=gradient
                        range_y=range_y />
                </Show>
                <Show when=move || fill_above.get().is_some()>
                    <clipPath id=clip_above_id.clone()>
                        <path d=clip_above_path />
                    </clipPath>
                </Show>
                <Show when=move || fill_below.get().is_some()>
                    <clipPath id=clip_below_id.clone()>
                        <path d=clip_below_path />
                    </clipPath>
                </Show>
            </defs>
            <Show when=move || fill_above.get().is_some()>
                {
                    let bounds = inner_bounds;
                    let clip_url = clip_above_url.clone();
                    move || bounds.map(|b| {
                        let b = b.get();
                        view! {
                            <rect
                                clip-path=clip_url.clone()
                                x=b.left_x()
                                y=b.top_y()
                                width=b.width()
                                height=b.height()
                                fill=move || fill_above.get().map(|c| c.to_string()).unwrap_or_default()
                                stroke="none" />
                        }
                    })
                }
            </Show>
            <Show when=move || fill_below.get().is_some()>
                {
                    let bounds = inner_bounds;
                    let clip_url = clip_below_url.clone();
                    move || bounds.map(|b| {
                        let b = b.get();
                        view! {
                            <rect
                                clip-path=clip_url.clone()
                                x=b.left_x()
                                y=b.top_y()
                                width=b.width()
                                height=b.height()
                                fill=move || fill_below.get().map(|c| c.to_string()).unwrap_or_default()
                                stroke="none" />
                        }
                    })
                }
            </Show>
            <path d=path fill="none" />
            <marker::LineMarkers line=line positions=markers />
        </g>
    }
    .into_any()
}

/// Build a closed polygon path for clipping.
/// `above`: true for the region above the line (higher Y data values), false for below.
fn build_clip_path(
    positions: &[(f64, f64)],
    bounds: &Bounds,
    x_is_horizontal: bool,
    above: bool,
    interpolation: Interpolation,
) -> String {
    // Filter out NaN positions
    let valid: Vec<(f64, f64)> = positions
        .iter()
        .copied()
        .filter(|(x, y)| !x.is_nan() && !y.is_nan())
        .collect();
    if valid.is_empty() {
        return String::new();
    }

    let line_path = interpolation.path(&valid, x_is_horizontal);
    // Replace leading "M" with "L" so the line path continues from our polygon start
    let line_as_lineto = format!("L{}", &line_path[1..]);

    let first = valid.first().unwrap();
    let last = valid.last().unwrap();

    // Build corner points that extend from the first/last line points perpendicular
    // to the data direction, toward the fill edge. The start_corner is at the same
    // chart edge as the first point, and end_corner at the same edge as the last point.
    // This avoids diagonal lines that would cross the data line.
    let (start_corner, end_corner) = if x_is_horizontal {
        // X is horizontal: first/last differ in SVG x, fill edge is top or bottom
        let edge_y = if above { bounds.top_y() } else { bounds.bottom_y() };
        (
            format!("{},{}", first.0, edge_y),
            format!("{},{}", last.0, edge_y),
        )
    } else {
        // X is vertical: first/last differ in SVG y, fill edge is right or left
        let edge_x = if above { bounds.right_x() } else { bounds.left_x() };
        (
            format!("{},{}", edge_x, first.1),
            format!("{},{}", edge_x, last.1),
        )
    };

    // Polygon: start_corner → line path (as lineto) → end_corner → close
    format!("M {start_corner} {line_as_lineto} L {end_corner} Z")
}
