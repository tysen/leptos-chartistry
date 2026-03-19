mod aligned_floats;
mod aligned_integers;
mod span;
#[cfg(feature = "timestamps")]
mod timestamps;

pub use aligned_floats::AlignedFloats;
pub use aligned_integers::AlignedIntegers;
pub use span::{HorizontalSpan, TickFormatFn, VerticalSpan};
#[cfg(feature = "timestamps")]
pub use timestamps::{Period, Timestamps};

use std::sync::Arc;

/// Generates ticks for an axis given a range and available space.
pub trait Generator: Send + Sync {
    /// The tick type produced by this generator.
    type Tick;

    /// Generate ticks between `first` and `last` that fit within the given `span`.
    fn generate(
        &self,
        first: &Self::Tick,
        last: &Self::Tick,
        span: &dyn Span<Self::Tick>,
    ) -> GeneratedTicks<Self::Tick>;
}

/// Measures how much space ticks consume along an axis.
pub trait Span<Tick> {
    /// Total available length along the axis in pixels.
    fn length(&self) -> f64;
    /// Length consumed by the given ticks when formatted with `state`.
    fn consumed(&self, state: &dyn Format<Tick = Tick>, ticks: &[Tick]) -> f64;
}

/// Formats a tick value into a string. The precise format will be picked by the tick generator. For example if [Timestamps] is used and is only showing years then the format will be `YYYY`.
pub trait Format {
    /// Our tick value.
    type Tick;

    /// Formats a tick into a string according to the tick generator used.
    fn format(&self, value: &Self::Tick) -> String;
}

/// The result of generating ticks: the ticks themselves and the format state used to render them.
#[derive(Clone)]
#[non_exhaustive]
pub struct GeneratedTicks<Tick> {
    /// Format state chosen by the generator (e.g., decimal places, date granularity).
    pub state: Arc<dyn Format<Tick = Tick> + Send + Sync>,
    /// The generated tick values.
    pub ticks: Vec<Tick>,
}

impl<Tick> GeneratedTicks<Tick> {
    /// Creates a new `GeneratedTicks` from a format state and tick values.
    pub fn new(state: impl Format<Tick = Tick> + Send + Sync + 'static, ticks: Vec<Tick>) -> Self {
        GeneratedTicks {
            state: Arc::new(state),
            ticks,
        }
    }
}

impl<Tick: Send + Sync + 'static> GeneratedTicks<Tick> {
    /// Creates an empty `GeneratedTicks` with no ticks.
    pub fn none() -> GeneratedTicks<Tick> {
        Self::new(NilState(std::marker::PhantomData), vec![])
    }
}

// Dummy TickState that should never be called. Used with no ticks.
struct NilState<Tick>(std::marker::PhantomData<Tick>);

impl<Tick> Format for NilState<Tick> {
    type Tick = Tick;

    fn format(&self, _: &Self::Tick) -> String {
        "-".to_string()
    }
}

/// Note: PartialEq only compares the `ticks`. Meaning TickGen implementations must result in the same TickState when Ticks are equal.
impl<Tick: PartialEq> PartialEq for GeneratedTicks<Tick> {
    fn eq(&self, other: &Self) -> bool {
        self.ticks == other.ticks
    }
}
