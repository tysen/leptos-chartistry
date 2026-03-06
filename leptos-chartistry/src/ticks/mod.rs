mod gen;

pub use gen::{
    AlignedFloats, AlignedIntegers, Format as TickFormat, GeneratedTicks, Generator as TickGen,
    HorizontalSpan, Period, TickFormatFn, Timestamps, VerticalSpan,
};

use chrono::prelude::*;

mod private {
    pub trait Sealed {}
}

/// A type that can be used as a tick on an axis. Try to rely on provided implementations.
pub trait Tick: Clone + PartialEq + PartialOrd + Send + Sync + 'static + private::Sealed {
    /// Default tick generator used in tick labels.
    fn tick_label_generator() -> impl TickGen<Tick = Self>;

    /// Default tick generator used in tooltips.
    fn tooltip_generator() -> impl TickGen<Tick = Self> {
        Self::tick_label_generator()
    }

    /// Maps the tick to a position on the axis. Must be uniform. May return `f64::NAN` for missing data.
    fn position(&self) -> f64;
}

impl private::Sealed for f64 {}
impl<Tz: TimeZone> private::Sealed for DateTime<Tz> {}

impl Tick for f64 {
    fn tick_label_generator() -> impl TickGen<Tick = Self> {
        AlignedFloats::default()
    }

    fn position(&self) -> f64 {
        *self
    }
}

impl<Tz> Tick for DateTime<Tz>
where
    Tz: TimeZone + Send + Sync + 'static,
    Tz::Offset: std::fmt::Display + Send + Sync,
{
    fn tick_label_generator() -> impl TickGen<Tick = Self> {
        Timestamps::default()
    }

    fn tooltip_generator() -> impl TickGen<Tick = Self> {
        Timestamps::default().with_long_format()
    }

    fn position(&self) -> f64 {
        self.timestamp() as f64 + (self.timestamp_subsec_nanos() as f64 / 1e9)
    }
}

/// Macro to implement Tick for integer types.
/// Note: For very large integers (i64::MAX, u64::MAX), casting to f64 loses precision.
/// This is a known limitation also present in DateTime's implementation.
macro_rules! impl_tick_for_integer {
    ($($t:ty),*) => {
        $(
            impl private::Sealed for $t {}

            impl Tick for $t {
                fn tick_label_generator() -> impl TickGen<Tick = Self> {
                    AlignedIntegers::<$t>::default()
                }

                fn position(&self) -> f64 {
                    *self as f64
                }
            }
        )*
    };
}

impl_tick_for_integer!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
