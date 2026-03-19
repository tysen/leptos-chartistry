use super::{Format, GeneratedTicks, Generator, Span};
use std::marker::PhantomData;

/// Generates integer ticks aligned to "nice" values (1, 2, 5, 10, 20, 50, 100...).
///
/// This generator works entirely in the integer domain, producing clean integer
/// labels without decimal points. For axes representing discrete values like
/// scores, counts, or indices, this ensures labels like `[-2, -1, 0, 1, 2]`
/// rather than `[-1.5, 0.0, 1.5]`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct AlignedIntegers<T> {
    _marker: PhantomData<T>,
}

/// Format state for integer ticks - simply converts to string.
#[derive(Clone, Debug, PartialEq)]
struct IntegerFormat<T> {
    _marker: PhantomData<T>,
}

impl<T> IntegerFormat<T> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: std::fmt::Display> Format for IntegerFormat<T> {
    type Tick = T;

    fn format(&self, value: &Self::Tick) -> String {
        value.to_string()
    }
}

/// Nice step sizes follow the pattern: 1, 2, 5 at each power of 10.
/// Given a range and approximate max tick count, find the smallest nice step
/// that produces at most max_ticks ticks.
fn find_nice_step(range: u128, max_ticks: usize) -> u128 {
    if range == 0 || max_ticks <= 1 {
        return 1;
    }

    // Calculate minimum step needed
    let min_step = (range / max_ticks as u128).max(1);

    // Find the nice step >= min_step
    // Nice steps are: 1, 2, 5, 10, 20, 50, 100, 200, 500, ...
    let mut power: u128 = 1;
    loop {
        for &multiplier in &[1u128, 2, 5] {
            let step = power * multiplier;
            if step >= min_step {
                return step;
            }
        }
        // Prevent overflow
        if power > u128::MAX / 10 {
            return power * 5;
        }
        power *= 10;
    }
}

/// Calculate approximate max ticks that fit in the span.
fn max_ticks_from_span<T: std::fmt::Display>(first: &T, last: &T, span: &dyn Span<T>) -> usize {
    let state = IntegerFormat::<T>::new();
    // Estimate consumed width from first and last values
    let first_consumed = span.consumed(&state, std::slice::from_ref(first));
    let last_consumed = span.consumed(&state, std::slice::from_ref(last));
    let consumed = first_consumed.max(last_consumed);
    if consumed <= 0.0 {
        return 10; // Fallback
    }
    (span.length() / consumed).max(2.0) as usize
}

/// Type-specific operations for integer tick generation.
trait IntegerTick: Copy + Ord + std::fmt::Display + Send + Sync + 'static {
    /// Compute the unsigned range between two ordered values.
    fn unsigned_range(lo: Self, hi: Self) -> u128;
    /// Convert a u128 step size to Self. The value is guaranteed to fit within the data range.
    fn from_u128(value: u128) -> Self;
    /// Align `lo` down to the nearest multiple of `step`.
    fn align_start(lo: Self, step: Self) -> Self;
    /// Checked addition.
    fn checked_add(self, step: Self) -> Option<Self>;
}

macro_rules! impl_integer_tick_signed {
    ($($t:ty),*) => {
        $(
            impl IntegerTick for $t {
                fn unsigned_range(lo: Self, hi: Self) -> u128 {
                    (hi as i128 - lo as i128).unsigned_abs()
                }
                fn from_u128(value: u128) -> Self {
                    value as $t
                }
                fn align_start(lo: Self, step: Self) -> Self {
                    if lo >= 0 {
                        (lo / step) * step
                    } else {
                        ((lo - step + 1) / step) * step
                    }
                }
                fn checked_add(self, step: Self) -> Option<Self> {
                    self.checked_add(step)
                }
            }
        )*
    };
}

macro_rules! impl_integer_tick_unsigned {
    ($($t:ty),*) => {
        $(
            impl IntegerTick for $t {
                fn unsigned_range(lo: Self, hi: Self) -> u128 {
                    (hi - lo) as u128
                }
                fn from_u128(value: u128) -> Self {
                    value as $t
                }
                fn align_start(lo: Self, step: Self) -> Self {
                    (lo / step) * step
                }
                fn checked_add(self, step: Self) -> Option<Self> {
                    self.checked_add(step)
                }
            }
        )*
    };
}

impl_integer_tick_signed!(i8, i16, i32, i64, i128, isize);
impl_integer_tick_unsigned!(u8, u16, u32, u64, u128, usize);

fn generate_ticks<T: IntegerTick>(first: T, last: T, span: &dyn Span<T>) -> GeneratedTicks<T> {
    // Handle zero range
    if first == last {
        return GeneratedTicks::new(IntegerFormat::new(), vec![first]);
    }

    // Ensure first <= last for calculation
    let (lo, hi) = if first <= last { (first, last) } else { (last, first) };
    let range = T::unsigned_range(lo, hi);

    let max_ticks = max_ticks_from_span(&lo, &hi, span);
    let step = T::from_u128(find_nice_step(range, max_ticks));

    let start = T::align_start(lo, step);

    // Generate ticks
    let mut ticks = Vec::default();
    let mut current = start;
    while current <= hi {
        if current >= lo {
            ticks.push(current);
        }
        if let Some(next) = current.checked_add(step) {
            current = next;
        } else {
            break;
        }
    }

    // Ensure we have at least one tick
    if ticks.is_empty() {
        ticks.push(lo);
    }

    GeneratedTicks::new(IntegerFormat::new(), ticks)
}

macro_rules! impl_generator {
    ($($t:ty),*) => {
        $(
            impl Generator for AlignedIntegers<$t> {
                type Tick = $t;

                fn generate(
                    &self,
                    first: &Self::Tick,
                    last: &Self::Tick,
                    span: &dyn Span<Self::Tick>,
                ) -> GeneratedTicks<Self::Tick> {
                    generate_ticks(*first, *last, span)
                }
            }
        )*
    };
}

impl_generator!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {
    use super::super::HorizontalSpan;
    use super::*;

    fn mk_span<T: crate::Tick>(width: f64) -> Box<dyn Span<T>> {
        Box::new(HorizontalSpan::new(
            1.0,
            0,
            0.0,
            width,
            HorizontalSpan::identity_format(),
        ))
    }

    fn generate_i64(first: i64, last: i64, width: f64) -> Vec<i64> {
        let gen = AlignedIntegers::<i64>::default();
        let span = mk_span::<i64>(width);
        gen.generate(&first, &last, span.as_ref()).ticks
    }

    fn generate_u64(first: u64, last: u64, width: f64) -> Vec<u64> {
        let gen = AlignedIntegers::<u64>::default();
        let span = mk_span::<u64>(width);
        gen.generate(&first, &last, span.as_ref()).ticks
    }

    #[test]
    fn test_find_nice_step() {
        // Small ranges
        assert_eq!(find_nice_step(5, 10), 1);
        assert_eq!(find_nice_step(10, 5), 2);
        assert_eq!(find_nice_step(10, 3), 5);
        assert_eq!(find_nice_step(10, 2), 5);

        // Medium ranges
        assert_eq!(find_nice_step(100, 20), 5);
        assert_eq!(find_nice_step(100, 10), 10);
        assert_eq!(find_nice_step(100, 5), 20);
        assert_eq!(find_nice_step(100, 3), 50);

        // Large ranges
        assert_eq!(find_nice_step(1000, 10), 100);
        assert_eq!(find_nice_step(1000, 5), 200);
        assert_eq!(find_nice_step(1000, 3), 500);

        // Edge cases
        assert_eq!(find_nice_step(0, 10), 1);
        assert_eq!(find_nice_step(100, 0), 1);
        assert_eq!(find_nice_step(100, 1), 1);
    }

    #[test]
    fn test_small_range_signed() {
        // Range -2 to 5 with large space should include all values
        let ticks = generate_i64(-2, 5, 100.0);
        assert!(!ticks.is_empty());
        assert!(ticks.iter().all(|&t| t >= -2 && t <= 5));
        // Verify step is a nice number
        if ticks.len() > 1 {
            let step = ticks[1] - ticks[0];
            assert!(step == 1 || step == 2 || step == 5, "step was {}", step);
        }

        // Range -2 to 5 with small space should give fewer ticks
        let ticks = generate_i64(-2, 5, 10.0);
        assert!(ticks.len() >= 2);
        assert!(ticks.iter().all(|&t| t >= -2 && t <= 5));
    }

    #[test]
    fn test_small_range_unsigned() {
        // Range 0 to 10 with large space
        let ticks = generate_u64(0, 10, 100.0);
        assert!(!ticks.is_empty());
        assert!(ticks.iter().all(|&t| t <= 10));
        // Verify step is a nice number
        if ticks.len() > 1 {
            let step = ticks[1] - ticks[0];
            assert!(step == 1 || step == 2 || step == 5, "step was {}", step);
        }

        // Range 0 to 10 with smaller space
        let ticks = generate_u64(0, 10, 20.0);
        assert!(ticks.len() >= 2);
        assert!(ticks.iter().all(|&t| t <= 10));
    }

    #[test]
    fn test_medium_range() {
        // Range 0 to 100 with medium space
        let ticks = generate_i64(0, 100, 100.0);
        assert!(!ticks.is_empty());
        assert!(ticks.first().unwrap() >= &0);
        assert!(ticks.last().unwrap() <= &100);
        // Verify step is a nice number
        if ticks.len() > 1 {
            let step = ticks[1] - ticks[0];
            assert!(
                step == 1
                    || step == 2
                    || step == 5
                    || step == 10
                    || step == 20
                    || step == 25
                    || step == 50
                    || step == 100,
                "step was {}",
                step
            );
        }
    }

    #[test]
    fn test_large_range() {
        // Range 0 to 1000000 with small space
        let ticks = generate_i64(0, 1_000_000, 50.0);
        assert!(ticks.len() >= 2 && ticks.len() <= 10);
        if ticks.len() > 1 {
            let step = ticks[1] - ticks[0];
            // Step should be a nice number (100000, 200000, 500000, etc.)
            assert!(
                step == 100_000 || step == 200_000 || step == 500_000 || step == 1_000_000,
                "step was {}",
                step
            );
        }
    }

    #[test]
    fn test_negative_range() {
        // Entirely negative range
        let ticks = generate_i64(-100, -10, 50.0);
        assert!(ticks.iter().all(|&t| t >= -100 && t <= -10));
        assert!(!ticks.is_empty());

        // Large negative range
        let ticks = generate_i64(-1000, -500, 50.0);
        if ticks.len() > 1 {
            let step = ticks[1] - ticks[0];
            assert!(
                step == 50 || step == 100 || step == 200,
                "step was {}",
                step
            );
        }
    }

    #[test]
    fn test_zero_range() {
        // Single value range
        let ticks = generate_i64(42, 42, 100.0);
        assert_eq!(ticks, vec![42]);

        let ticks = generate_u64(100, 100, 100.0);
        assert_eq!(ticks, vec![100]);
    }

    #[test]
    fn test_reversed_range() {
        // Handles reversed first/last
        let ticks = generate_i64(10, 0, 100.0);
        assert!(!ticks.is_empty());
        assert!(ticks.iter().all(|&t| t >= 0 && t <= 10));
    }

    #[test]
    fn test_format() {
        let format = IntegerFormat::<i64>::new();
        assert_eq!(format.format(&0), "0");
        assert_eq!(format.format(&42), "42");
        assert_eq!(format.format(&-100), "-100");
        assert_eq!(format.format(&1_000_000), "1000000");

        let format = IntegerFormat::<u64>::new();
        assert_eq!(format.format(&0), "0");
        assert_eq!(format.format(&u64::MAX), "18446744073709551615");
    }

    #[test]
    fn test_all_integer_types() {
        // Verify all types compile and work
        let span_i8 = mk_span::<i8>(50.0);
        let span_i16 = mk_span::<i16>(50.0);
        let span_i32 = mk_span::<i32>(50.0);
        let span_i64 = mk_span::<i64>(50.0);
        let span_i128 = mk_span::<i128>(50.0);
        let span_isize = mk_span::<isize>(50.0);
        let span_u8 = mk_span::<u8>(50.0);
        let span_u16 = mk_span::<u16>(50.0);
        let span_u32 = mk_span::<u32>(50.0);
        let span_u64 = mk_span::<u64>(50.0);
        let span_u128 = mk_span::<u128>(50.0);
        let span_usize = mk_span::<usize>(50.0);

        assert!(!AlignedIntegers::<i8>::default()
            .generate(&0, &10, span_i8.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<i16>::default()
            .generate(&0, &10, span_i16.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<i32>::default()
            .generate(&0, &10, span_i32.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<i64>::default()
            .generate(&0, &10, span_i64.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<i128>::default()
            .generate(&0, &10, span_i128.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<isize>::default()
            .generate(&0, &10, span_isize.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<u8>::default()
            .generate(&0, &10, span_u8.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<u16>::default()
            .generate(&0, &10, span_u16.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<u32>::default()
            .generate(&0, &10, span_u32.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<u64>::default()
            .generate(&0, &10, span_u64.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<u128>::default()
            .generate(&0, &10, span_u128.as_ref())
            .ticks
            .is_empty());
        assert!(!AlignedIntegers::<usize>::default()
            .generate(&0, &10, span_usize.as_ref())
            .ticks
            .is_empty());
    }
}
