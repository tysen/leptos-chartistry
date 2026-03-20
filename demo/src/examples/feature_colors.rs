use super::MyData;
use leptos::prelude::*;
use leptos_chartistry::*;

#[component]
pub fn Example(debug: Signal<bool>, data: Signal<Vec<MyData>>) -> impl IntoView {
    // A variety of ways to create new colors
    const BLACK: Color = Color::from_rgb(0, 0, 0);
    const THISTLE: Color = Color::from_rgb(216, 191, 216);
    let sea_green: Color = "#20b2aa".parse().unwrap();
    const RED: Color = Color::from_rgb(255, 0, 0);
    const BLUE_VIOLET: Color = Color::from_rgb(0, 0, 255);

    // We can also describe a color scheme for our Series:
    // For non-stacked, colors are picked one after the other and then repeat
    // For stacked lines, colors are interpolated between the first and last
    let scheme = ColorScheme::new(RED, vec![BLUE_VIOLET]);

    // Add names to our lines for the legend to use
    let series = Series::new(|data: &MyData| data.x)
        .line(
            Line::new(|data: &MyData| data.y1)
                .with_name("roses")
                // Manually specify the color of a line
                .with_color(RED),
        )
        .line(
            Line::new(|data: &MyData| data.y2)
                .with_name("violets")
                .with_color(BLUE_VIOLET),
        )
        // Or specify the color scheme (this gives the same as above but more flexible)
        .with_colors(scheme);

    view! {
        <Chart
            aspect_ratio=AspectRatio::from_outer_height(300.0, 1.2)
            debug=debug
            series=series
            data=data

            inner=vec![
                // Most drawn elements can have their color changed
                AxisMarker::left_edge().with_color(BLACK).into_inner(),
                YGridLine::default().with_color(THISTLE).into_inner(),
                YGuideLine::over_mouse().with_color(sea_green).into_inner(),
                // Legends pick their colors from the lines they're describing
                InsetLegend::bottom_right().into_inner(),
            ]
        />
    }
}
