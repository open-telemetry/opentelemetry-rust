use std::{collections::HashMap, f64::consts::LOG2_E, sync::Mutex, time::SystemTime};

use once_cell::sync::Lazy;
use opentelemetry::{attributes::AttributeSet, metrics::MetricsError};

use crate::metrics::data::{self, Aggregation, Temporality};

use super::Number;

pub(crate) const EXPO_MAX_SCALE: i8 = 20;
pub(crate) const EXPO_MIN_SCALE: i8 = -10;

/// A single data point in an exponential histogram.
#[derive(Debug, PartialEq)]
struct ExpoHistogramDataPoint<T> {
    count: usize,
    min: T,
    max: T,
    sum: T,

    max_size: i32,
    record_min_max: bool,
    record_sum: bool,

    scale: i8,

    pos_buckets: ExpoBuckets,
    neg_buckets: ExpoBuckets,
    zero_count: u64,
}

impl<T: Number<T>> ExpoHistogramDataPoint<T> {
    fn new(max_size: i32, max_scale: i8, record_min_max: bool, record_sum: bool) -> Self {
        ExpoHistogramDataPoint {
            count: 0,
            min: T::max(),
            max: T::min(),
            sum: T::default(),
            max_size,
            record_min_max,
            record_sum,
            scale: max_scale,
            pos_buckets: ExpoBuckets::default(),
            neg_buckets: ExpoBuckets::default(),
            zero_count: 0,
        }
    }
}

impl<T: Number<T>> ExpoHistogramDataPoint<T> {
    /// Adds a new measurement to the histogram.
    ///
    /// It will rescale the buckets if needed.
    fn record(&mut self, v: T) {
        self.count += 1;

        if self.record_min_max {
            if v < self.min {
                self.min = v;
            }
            if v > self.max {
                self.max = v;
            }
        }
        if self.record_sum {
            self.sum += v;
        }

        let abs_v = v.into_float().abs();

        if abs_v == 0.0 {
            self.zero_count += 1;
            return;
        }

        let mut bin = self.get_bin(abs_v);

        let v_is_negative = v < T::default();

        // If the new bin would make the counts larger than `max_scale`, we need to
        // downscale current measurements.
        let scale_delta = {
            let bucket = if v_is_negative {
                &self.neg_buckets
            } else {
                &self.pos_buckets
            };

            scale_change(
                self.max_size,
                bin,
                bucket.start_bin,
                bucket.counts.len() as i32,
            )
        };
        if scale_delta > 0 {
            if (self.scale - scale_delta as i8) < EXPO_MIN_SCALE {
                // With a scale of -10 there is only two buckets for the whole range of f64 values.
                // This can only happen if there is a max size of 1.
                opentelemetry::global::handle_error(MetricsError::Other(
                    "exponential histogram scale underflow".into(),
                ));
                return;
            }
            // Downscale
            self.scale -= scale_delta as i8;
            self.pos_buckets.downscale(scale_delta);
            self.neg_buckets.downscale(scale_delta);

            bin = self.get_bin(abs_v);
        }

        if v_is_negative {
            self.neg_buckets.record(bin)
        } else {
            self.pos_buckets.record(bin)
        }
    }

    /// the bin `v` should be recorded into.
    fn get_bin(&self, v: f64) -> i32 {
        let (frac, exp) = frexp(v);
        if self.scale <= 0 {
            // With negative scale `frac` is always 1 power of two higher than we want.
            let mut correction = 1;
            if frac == 0.5 {
                // If `v` is an exact power of two, `frac` will be `0.5` and the exp
                // will be then be two higher than we want.
                correction = 2;
            }
            return (exp - correction) >> -self.scale;
        }
        (exp << self.scale) + (frac.ln() * SCALE_FACTORS[self.scale as usize]) as i32 - 1
    }
}

/// The magnitude of the scale change needed to fit bin in the bucket.
///
/// If no scale change is needed 0 is returned.
fn scale_change(max_size: i32, bin: i32, start_bin: i32, length: i32) -> u32 {
    if length == 0 {
        // No need to rescale if there are no buckets.
        return 0;
    }

    let mut low = start_bin;
    let mut high = bin;
    if start_bin >= bin {
        low = bin;
        high = start_bin + length - 1;
    }

    let mut count = 0u32;
    while high - low >= max_size {
        low >>= 1;
        high >>= 1;
        count += 1;

        if count > (EXPO_MAX_SCALE - EXPO_MIN_SCALE) as u32 {
            return count;
        }
    }

    count
}

/// Constants used in calculating the logarithm index.
static SCALE_FACTORS: Lazy<[f64; 21]> = Lazy::new(|| {
    [
        LOG2_E * 2f64.powi(0),
        LOG2_E * 2f64.powi(1),
        LOG2_E * 2f64.powi(2),
        LOG2_E * 2f64.powi(3),
        LOG2_E * 2f64.powi(4),
        LOG2_E * 2f64.powi(5),
        LOG2_E * 2f64.powi(6),
        LOG2_E * 2f64.powi(7),
        LOG2_E * 2f64.powi(8),
        LOG2_E * 2f64.powi(9),
        LOG2_E * 2f64.powi(10),
        LOG2_E * 2f64.powi(11),
        LOG2_E * 2f64.powi(12),
        LOG2_E * 2f64.powi(13),
        LOG2_E * 2f64.powi(14),
        LOG2_E * 2f64.powi(15),
        LOG2_E * 2f64.powi(16),
        LOG2_E * 2f64.powi(17),
        LOG2_E * 2f64.powi(18),
        LOG2_E * 2f64.powi(19),
        LOG2_E * 2f64.powi(20),
    ]
});

/// Breaks the number into a normalized fraction and a base-2 exponent.
///
/// This impl is necessary as rust removed this functionality from std in
/// <https://github.com/rust-lang/rust/pull/41437>
#[inline(always)]
fn frexp(x: f64) -> (f64, i32) {
    let mut y = x.to_bits();
    let ee = ((y >> 52) & 0x7ff) as i32;

    if ee == 0 {
        if x != 0.0 {
            let x1p64 = f64::from_bits(0x43f0000000000000);
            let (x, e) = frexp(x * x1p64);
            return (x, e - 64);
        }
        return (x, 0);
    } else if ee == 0x7ff {
        return (x, 0);
    }

    let e = ee - 0x3fe;
    y &= 0x800fffffffffffff;
    y |= 0x3fe0000000000000;

    (f64::from_bits(y), e)
}

/// A set of buckets in an exponential histogram.
#[derive(Default, Debug, PartialEq)]
struct ExpoBuckets {
    start_bin: i32,
    counts: Vec<u64>,
}

impl ExpoBuckets {
    /// Increments the count for the given bin, and expands the buckets if needed.
    ///
    /// Size changes must be done before calling this function.
    fn record(&mut self, bin: i32) {
        if self.counts.is_empty() {
            self.counts = vec![1];
            self.start_bin = bin;
            return;
        }

        let end_bin = self.start_bin + self.counts.len() as i32 - 1;

        // if the new bin is inside the current range
        if bin >= self.start_bin && bin <= end_bin {
            self.counts[(bin - self.start_bin) as usize] += 1;
            return;
        }

        // if the new bin is before the current start, prepend the slots in `self.counts`
        if bin < self.start_bin {
            let mut zeroes = vec![0; (end_bin - bin + 1) as usize];
            let shift = (self.start_bin - bin) as usize;
            zeroes[shift..].copy_from_slice(&self.counts);
            self.counts = zeroes;
            self.counts[0] = 1;
            self.start_bin = bin;
        } else if bin > end_bin {
            // if the new bin is after the end, initialize the slots up to the new bin
            if ((bin - self.start_bin) as usize) < self.counts.capacity() {
                self.counts.resize((bin - self.start_bin + 1) as usize, 0);
                self.counts[(bin - self.start_bin) as usize] = 1;
                return;
            }

            self.counts.extend(
                std::iter::repeat(0).take((bin - self.start_bin) as usize - self.counts.len() + 1),
            );
            self.counts[(bin - self.start_bin) as usize] = 1
        }
    }

    /// Shrinks a bucket by a factor of 2*s.
    ///
    /// It will sum counts into the correct lower resolution bucket.
    fn downscale(&mut self, delta: u32) {
        // Example
        // delta = 2
        // original offset: -6
        // counts: [ 3,  1,  2,  3,  4,  5, 6, 7, 8, 9, 10]
        // bins:    -6  -5, -4, -3, -2, -1, 0, 1, 2, 3, 4
        // new bins:-2, -2, -1, -1, -1, -1, 0, 0, 0, 0, 1
        // new offset: -2
        // new counts: [4, 14, 30, 10]

        if self.counts.len() <= 1 || delta < 1 {
            self.start_bin >>= delta;
            return;
        }

        let steps = 1 << delta;
        let mut offset = self.start_bin % steps;
        offset = (offset + steps) % steps; // to make offset positive
        for i in 1..self.counts.len() {
            let idx = i + offset as usize;
            if idx % steps as usize == 0 {
                self.counts[idx / steps as usize] = self.counts[i];
                continue;
            }
            self.counts[idx / steps as usize] += self.counts[i];
        }

        let last_idx = (self.counts.len() as i32 - 1 + offset) / steps;
        self.counts = self.counts[..last_idx as usize + 1].to_vec();
        self.start_bin >>= delta;
    }
}

/// An aggregator that summarizes a set of measurements as an exponential
/// histogram.
///
/// Each histogram is scoped by attributes and the aggregation cycle the
/// measurements were made in.
pub(crate) struct ExpoHistogram<T> {
    record_sum: bool,
    record_min_max: bool,
    max_size: i32,
    max_scale: i8,

    values: Mutex<HashMap<AttributeSet, ExpoHistogramDataPoint<T>>>,

    start: Mutex<SystemTime>,
}

impl<T: Number<T>> ExpoHistogram<T> {
    /// Create a new exponential histogram.
    pub(crate) fn new(
        max_size: u32,
        max_scale: i8,
        record_min_max: bool,
        record_sum: bool,
    ) -> Self {
        ExpoHistogram {
            record_sum,
            record_min_max,
            max_size: max_size as i32,
            max_scale,
            values: Mutex::new(HashMap::default()),
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, value: T, attrs: AttributeSet) {
        let f_value = value.into_float();
        // Ignore NaN and infinity.
        if f_value.is_infinite() || f_value.is_nan() {
            return;
        }

        if let Ok(mut values) = self.values.lock() {
            let v = values.entry(attrs).or_insert_with(|| {
                ExpoHistogramDataPoint::new(
                    self.max_size,
                    self.max_scale,
                    self.record_min_max,
                    self.record_sum,
                )
            });
            v.record(value)
        }
    }

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();
        let start = self
            .start
            .lock()
            .map(|s| *s)
            .unwrap_or_else(|_| SystemTime::now());

        let h = dest.and_then(|d| d.as_mut().downcast_mut::<data::ExponentialHistogram<T>>());
        let mut new_agg = if h.is_none() {
            Some(data::ExponentialHistogram {
                data_points: vec![],
                temporality: Temporality::Delta,
            })
        } else {
            None
        };
        let h = h.unwrap_or_else(|| new_agg.as_mut().expect("present if h is none"));
        h.temporality = Temporality::Delta;
        h.data_points.clear();

        let mut values = match self.values.lock() {
            Ok(g) => g,
            Err(_) => return (0, None),
        };

        let n = values.len();
        if n > h.data_points.capacity() {
            h.data_points.reserve_exact(n - h.data_points.capacity());
        }

        for (a, b) in values.drain() {
            h.data_points.push(data::ExponentialHistogramDataPoint {
                attributes: a,
                start_time: start,
                time: t,
                count: b.count,
                min: if self.record_min_max {
                    Some(b.min)
                } else {
                    None
                },
                max: if self.record_min_max {
                    Some(b.max)
                } else {
                    None
                },
                sum: if self.record_sum { b.sum } else { T::default() },
                scale: b.scale,
                zero_count: b.zero_count,
                positive_bucket: data::ExponentialBucket {
                    offset: b.pos_buckets.start_bin,
                    counts: b.pos_buckets.counts.clone(),
                },
                negative_bucket: data::ExponentialBucket {
                    offset: b.neg_buckets.start_bin,
                    counts: b.neg_buckets.counts.clone(),
                },
                zero_threshold: 0.0,
                exemplars: vec![],
            });
        }

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }

        (n, new_agg.map(|a| Box::new(a) as Box<_>))
    }

    pub(crate) fn cumulative(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();
        let start = self
            .start
            .lock()
            .map(|s| *s)
            .unwrap_or_else(|_| SystemTime::now());

        let h = dest.and_then(|d| d.as_mut().downcast_mut::<data::ExponentialHistogram<T>>());
        let mut new_agg = if h.is_none() {
            Some(data::ExponentialHistogram {
                data_points: vec![],
                temporality: Temporality::Cumulative,
            })
        } else {
            None
        };
        let h = h.unwrap_or_else(|| new_agg.as_mut().expect("present if h is none"));
        h.temporality = Temporality::Cumulative;

        let values = match self.values.lock() {
            Ok(g) => g,
            Err(_) => return (0, None),
        };
        h.data_points.clear();

        let n = values.len();
        if n > h.data_points.capacity() {
            h.data_points.reserve_exact(n - h.data_points.capacity());
        }

        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        for (a, b) in values.iter() {
            h.data_points.push(data::ExponentialHistogramDataPoint {
                attributes: a.clone(),
                start_time: start,
                time: t,
                count: b.count,
                min: if self.record_min_max {
                    Some(b.min)
                } else {
                    None
                },
                max: if self.record_min_max {
                    Some(b.max)
                } else {
                    None
                },
                sum: if self.record_sum { b.sum } else { T::default() },
                scale: b.scale,
                zero_count: b.zero_count,
                positive_bucket: data::ExponentialBucket {
                    offset: b.pos_buckets.start_bin,
                    counts: b.pos_buckets.counts.clone(),
                },
                negative_bucket: data::ExponentialBucket {
                    offset: b.neg_buckets.start_bin,
                    counts: b.neg_buckets.counts.clone(),
                },
                zero_threshold: 0.0,
                exemplars: vec![],
            });
        }

        (n, new_agg.map(|a| Box::new(a) as Box<_>))
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Neg;

    use opentelemetry::KeyValue;

    use crate::metrics::internal::{self, AggregateBuilder};

    use super::*;

    #[test]
    fn test_expo_histogram_data_point_record() {
        run_data_point_record::<f64>();
        run_data_point_record_f64();
        run_min_max_sum_f64();
        run_min_max_sum::<i64>();
        run_min_max_sum::<u64>();
        run_data_point_record::<i64>();
    }

    fn run_data_point_record<T: Number<T> + Neg<Output = T> + From<u32>>() {
        struct TestCase<T> {
            max_size: i32,
            values: Vec<T>,
            expected_buckets: ExpoBuckets,
            expected_scale: i8,
        }
        let test_cases: Vec<TestCase<T>> = vec![
            TestCase {
                max_size: 4,
                values: vec![2, 4, 1].into_iter().map(Into::into).collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 1, 1],
                },
                expected_scale: 0,
            },
            TestCase {
                max_size: 4,
                values: vec![4, 4, 4, 2, 16, 1]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 4, 1],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![1, 2, 4].into_iter().map(Into::into).collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 2],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![1, 4, 2].into_iter().map(Into::into).collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 2],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![2, 4, 1].into_iter().map(Into::into).collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 2],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![2, 1, 4].into_iter().map(Into::into).collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 2],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![4, 1, 2].into_iter().map(Into::into).collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 2],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![4, 2, 1].into_iter().map(Into::into).collect(),
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 2],
                },
                expected_scale: -1,
            },
        ];

        for test in test_cases {
            let mut dp = ExpoHistogramDataPoint::<T>::new(test.max_size, 20, true, true);
            for v in test.values {
                dp.record(v);
                dp.record(-v);
            }

            assert_eq!(test.expected_buckets, dp.pos_buckets, "positive buckets");
            assert_eq!(test.expected_buckets, dp.neg_buckets, "negative buckets");
            assert_eq!(test.expected_scale, dp.scale, "scale");
        }
    }

    fn run_min_max_sum_f64() {
        let alice = AttributeSet::from(&[KeyValue::new("user", "alice")]);
        struct Expected {
            min: f64,
            max: f64,
            sum: f64,
            count: usize,
        }
        impl Expected {
            fn new(min: f64, max: f64, sum: f64, count: usize) -> Self {
                Expected {
                    min,
                    max,
                    sum,
                    count,
                }
            }
        }
        struct TestCase {
            values: Vec<f64>,
            expected: Expected,
        }

        let test_cases = vec![
            TestCase {
                values: vec![2.0, 4.0, 1.0],
                expected: Expected::new(1.0, 4.0, 7.0, 3),
            },
            TestCase {
                values: vec![2.0, 4.0, 1.0, f64::INFINITY],
                expected: Expected::new(1.0, 4.0, 7.0, 3),
            },
            TestCase {
                values: vec![2.0, 4.0, 1.0, -f64::INFINITY],
                expected: Expected::new(1.0, 4.0, 7.0, 3),
            },
            TestCase {
                values: vec![2.0, 4.0, 1.0, f64::NAN],
                expected: Expected::new(1.0, 4.0, 7.0, 3),
            },
            TestCase {
                values: vec![4.0, 4.0, 4.0, 2.0, 16.0, 1.0],
                expected: Expected::new(1.0, 16.0, 31.0, 6),
            },
        ];

        for test in test_cases {
            let h = ExpoHistogram::new(4, 20, true, true);
            for v in test.values {
                h.measure(v, alice.clone());
            }
            let values = h.values.lock().unwrap();
            let dp = values.get(&alice).unwrap();

            assert_eq!(test.expected.max, dp.max);
            assert_eq!(test.expected.min, dp.min);
            assert_eq!(test.expected.sum, dp.sum);
            assert_eq!(test.expected.count, dp.count);
        }
    }

    fn run_min_max_sum<T: Number<T> + From<u32>>() {
        let alice = AttributeSet::from(&[KeyValue::new("user", "alice")]);
        struct Expected<T> {
            min: T,
            max: T,
            sum: T,
            count: usize,
        }
        impl<T: Number<T>> Expected<T> {
            fn new(min: T, max: T, sum: T, count: usize) -> Self {
                Expected {
                    min,
                    max,
                    sum,
                    count,
                }
            }
        }
        struct TestCase<T> {
            values: Vec<T>,
            expected: Expected<T>,
        }
        let test_cases: Vec<TestCase<T>> = vec![
            TestCase {
                values: vec![2, 4, 1].into_iter().map(Into::into).collect(),
                expected: Expected::new(1.into(), 4.into(), 7.into(), 3),
            },
            TestCase {
                values: vec![4, 4, 4, 2, 16, 1]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                expected: Expected::new(1.into(), 16.into(), 31.into(), 6),
            },
        ];

        for test in test_cases {
            let h = ExpoHistogram::new(4, 20, true, true);
            for v in test.values {
                h.measure(v, alice.clone());
            }
            let values = h.values.lock().unwrap();
            let dp = values.get(&alice).unwrap();

            assert_eq!(test.expected.max, dp.max);
            assert_eq!(test.expected.min, dp.min);
            assert_eq!(test.expected.sum, dp.sum);
            assert_eq!(test.expected.count, dp.count);
        }
    }

    fn run_data_point_record_f64() {
        struct TestCase {
            max_size: i32,
            values: Vec<f64>,
            expected_buckets: ExpoBuckets,
            expected_scale: i8,
        }

        let test_cases = vec![
            TestCase {
                max_size: 4,
                values: vec![2.0, 2.0, 2.0, 1.0, 8.0, 0.5],
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![2, 3, 1],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![1.0, 0.5, 2.0],
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![2, 1],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![1.0, 2.0, 0.5],
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![2, 1],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![2.0, 0.5, 1.0],
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![2, 1],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![2.0, 1.0, 0.5],
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![2, 1],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![0.5, 1.0, 2.0],
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![2, 1],
                },
                expected_scale: -1,
            },
            TestCase {
                max_size: 2,
                values: vec![0.5, 2.0, 1.0],
                expected_buckets: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![2, 1],
                },
                expected_scale: -1,
            },
        ];
        for test in test_cases {
            let mut dp = ExpoHistogramDataPoint::new(test.max_size, 20, true, true);
            for v in test.values {
                dp.record(v);
                dp.record(-v);
            }

            assert_eq!(test.expected_buckets, dp.pos_buckets);
            assert_eq!(test.expected_buckets, dp.neg_buckets);
            assert_eq!(test.expected_scale, dp.scale);
        }
    }

    #[test]
    fn data_point_record_limits() {
        // These bins are calculated from the following formula:
        // floor( log2( value) * 2^20 ) using an arbitrary precision calculator.

        let mut fdp = ExpoHistogramDataPoint::new(4, 20, true, true);
        fdp.record(f64::MAX);

        assert_eq!(
            fdp.pos_buckets.start_bin, 1073741823,
            "start bin does not match for large f64 values",
        );

        let mut fdp = ExpoHistogramDataPoint::new(4, 20, true, true);
        fdp.record(f64::MIN_POSITIVE);

        assert_eq!(
            fdp.pos_buckets.start_bin, -1071644673,
            "start bin does not match for small positive values",
        );

        let mut idp = ExpoHistogramDataPoint::new(4, 20, true, true);
        idp.record(i64::MAX);

        assert_eq!(
            idp.pos_buckets.start_bin, 66060287,
            "start bin does not match for max i64 values",
        );
    }

    #[test]
    fn expo_bucket_downscale() {
        struct TestCase {
            name: &'static str,
            bucket: ExpoBuckets,
            scale: i8,
            want: ExpoBuckets,
        }

        let test_cases = vec![
            TestCase {
                name: "Empty bucket",
                bucket: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![],
                },
                scale: 3,
                want: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![],
                },
            },
            TestCase {
                name: "1 size bucket",
                bucket: ExpoBuckets {
                    start_bin: 50,
                    counts: vec![7],
                },
                scale: 4,
                want: ExpoBuckets {
                    start_bin: 3,
                    counts: vec![7],
                },
            },
            TestCase {
                name: "zero scale",
                bucket: ExpoBuckets {
                    start_bin: 50,
                    counts: vec![7, 5],
                },
                scale: 0,
                want: ExpoBuckets {
                    start_bin: 50,
                    counts: vec![7, 5],
                },
            },
            TestCase {
                name: "aligned bucket scale 1",
                bucket: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![1, 2, 3, 4, 5, 6],
                },
                scale: 1,
                want: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![3, 7, 11],
                },
            },
            TestCase {
                name: "aligned bucket scale 2",
                bucket: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![1, 2, 3, 4, 5, 6],
                },
                scale: 2,
                want: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![10, 11],
                },
            },
            TestCase {
                name: "aligned bucket scale 3",
                bucket: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![1, 2, 3, 4, 5, 6],
                },
                scale: 3,
                want: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![21],
                },
            },
            TestCase {
                name: "unaligned bucket scale 1",
                bucket: ExpoBuckets {
                    start_bin: 5,
                    counts: vec![1, 2, 3, 4, 5, 6],
                }, // This is equivalent to [0,0,0,0,0,1,2,3,4,5,6]
                scale: 1,
                want: ExpoBuckets {
                    start_bin: 2,
                    counts: vec![1, 5, 9, 6],
                }, // This is equivalent to [0,0,1,5,9,6]
            },
            TestCase {
                name: "unaligned bucket scale 2",
                bucket: ExpoBuckets {
                    start_bin: 7,
                    counts: vec![1, 2, 3, 4, 5, 6],
                }, // This is equivalent to [0,0,0,0,0,0,0,1,2,3,4,5,6]
                scale: 2,
                want: ExpoBuckets {
                    start_bin: 1,
                    counts: vec![1, 14, 6],
                }, // This is equivalent to [0,1,14,6]
            },
            TestCase {
                name: "unaligned bucket scale 3",
                bucket: ExpoBuckets {
                    start_bin: 3,
                    counts: vec![1, 2, 3, 4, 5, 6],
                }, // This is equivalent to [0,0,0,1,2,3,4,5,6]
                scale: 3,
                want: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![15, 6],
                }, // This is equivalent to [0,15,6]
            },
            TestCase {
                name: "unaligned bucket scale 1",
                bucket: ExpoBuckets {
                    start_bin: 1,
                    counts: vec![1, 0, 1],
                },
                scale: 1,
                want: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![1, 1],
                },
            },
            TestCase {
                name: "negative start_bin",
                bucket: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 0, 3],
                },
                scale: 1,
                want: ExpoBuckets {
                    start_bin: -1,
                    counts: vec![1, 3],
                },
            },
            TestCase {
                name: "negative start_bin 2",
                bucket: ExpoBuckets {
                    start_bin: -4,
                    counts: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                },
                scale: 1,
                want: ExpoBuckets {
                    start_bin: -2,
                    counts: vec![3, 7, 11, 15, 19],
                },
            },
        ];
        for mut test in test_cases {
            test.bucket.downscale(test.scale as u32);
            assert_eq!(test.want, test.bucket, "{}", test.name);
        }
    }

    #[test]
    fn expo_bucket_record() {
        struct TestCase {
            name: &'static str,
            bucket: ExpoBuckets,
            bin: i32,
            want: ExpoBuckets,
        }

        let test_cases = vec![
            TestCase {
                name: "Empty bucket creates first count",
                bucket: ExpoBuckets {
                    start_bin: 0,
                    counts: vec![],
                },
                bin: -5,
                want: ExpoBuckets {
                    start_bin: -5,
                    counts: vec![1],
                },
            },
            TestCase {
                name: "Bin is in the bucket",
                bucket: ExpoBuckets {
                    start_bin: 3,
                    counts: vec![1, 2, 3, 4, 5, 6],
                },
                bin: 5,
                want: ExpoBuckets {
                    start_bin: 3,
                    counts: vec![1, 2, 4, 4, 5, 6],
                },
            },
            TestCase {
                name: "Bin is before the start of the bucket",
                bucket: ExpoBuckets {
                    start_bin: 1,
                    counts: vec![1, 2, 3, 4, 5, 6],
                },
                bin: -2,
                want: ExpoBuckets {
                    start_bin: -2,
                    counts: vec![1, 0, 0, 1, 2, 3, 4, 5, 6],
                },
            },
            TestCase {
                name: "Bin is after the end of the bucket",
                bucket: ExpoBuckets {
                    start_bin: -2,
                    counts: vec![1, 2, 3, 4, 5, 6],
                },
                bin: 4,
                want: ExpoBuckets {
                    start_bin: -2,
                    counts: vec![1, 2, 3, 4, 5, 6, 1],
                },
            },
        ];

        for mut test in test_cases {
            test.bucket.record(test.bin);
            assert_eq!(test.want, test.bucket, "{}", test.name);
        }
    }

    #[test]
    fn scale_change_rescaling() {
        struct Args {
            bin: i32,
            start_bin: i32,
            length: i32,
            max_size: i32,
        }
        struct TestCase {
            name: &'static str,
            args: Args,
            want: u32,
        }
        let test_cases = vec![
            TestCase {
                name: "if length is 0, no rescale is needed",
                // [] -> [5] length 1
                args: Args {
                    bin: 5,
                    start_bin: 0,
                    length: 0,
                    max_size: 4,
                },
                want: 0,
            },
            TestCase {
                name: "if bin is between start, and the end, no rescale needed",
                // [-1, ..., 8] length 10 -> [-1, ..., 5, ..., 8] length 10
                args: Args {
                    bin: 5,
                    start_bin: -1,
                    length: 10,
                    max_size: 20,
                },
                want: 0,
            },
            TestCase {
                name: "if [bin,... end].len() > max_size, rescale needed",
                // [8,9,10] length 3 -> [5, ..., 10] length 6
                args: Args {
                    bin: 5,
                    start_bin: 8,
                    length: 3,
                    max_size: 5,
                },
                want: 1,
            },
            TestCase {
                name: "if [start, ..., bin].len() > max_size, rescale needed",
                // [2,3,4] length 3 -> [2, ..., 7] length 6
                args: Args {
                    bin: 7,
                    start_bin: 2,
                    length: 3,
                    max_size: 5,
                },
                want: 1,
            },
            TestCase {
                name: "if [start, ..., bin].len() > max_size, rescale needed",
                // [2,3,4] length 3 -> [2, ..., 7] length 12
                args: Args {
                    bin: 13,
                    start_bin: 2,
                    length: 3,
                    max_size: 5,
                },
                want: 2,
            },
            TestCase {
                name: "It should not hang if it will never be able to rescale",
                args: Args {
                    bin: 1,
                    start_bin: -1,
                    length: 1,
                    max_size: 1,
                },
                want: 31,
            },
        ];

        for test in test_cases {
            let got = scale_change(
                test.args.max_size,
                test.args.bin,
                test.args.start_bin,
                test.args.length,
            );
            assert_eq!(got, test.want, "incorrect scale change, {}", test.name);
        }
    }

    #[test]
    fn sub_normal() {
        let want = ExpoHistogramDataPoint {
            max_size: 4,
            count: 3,
            min: f64::MIN_POSITIVE,
            max: f64::MIN_POSITIVE,
            sum: 3.0 * f64::MIN_POSITIVE,

            scale: 20,
            pos_buckets: ExpoBuckets {
                start_bin: -1071644673,
                counts: vec![3],
            },
            neg_buckets: ExpoBuckets {
                start_bin: 0,
                counts: vec![],
            },
            record_min_max: true,
            record_sum: true,
            zero_count: 0,
        };

        let mut ehdp = ExpoHistogramDataPoint::new(4, 20, true, true);
        ehdp.record(f64::MIN_POSITIVE);
        ehdp.record(f64::MIN_POSITIVE);
        ehdp.record(f64::MIN_POSITIVE);

        assert_eq!(want, ehdp);
    }

    #[test]
    fn hist_aggregations() {
        hist_aggregation::<i64>();
        hist_aggregation::<u64>();
        hist_aggregation::<f64>();
    }

    fn box_val<T>(
        (m, ca): (impl internal::Measure<T>, impl internal::ComputeAggregation),
    ) -> (
        Box<dyn internal::Measure<T>>,
        Box<dyn internal::ComputeAggregation>,
    ) {
        (Box::new(m), Box::new(ca))
    }

    fn hist_aggregation<T: Number<T> + From<u32>>() {
        let max_size = 4;
        let max_scale = 20;
        let record_min_max = true;
        let record_sum = true;

        #[allow(clippy::type_complexity)]
        struct TestCase<T> {
            name: &'static str,
            build: Box<
                dyn Fn() -> (
                    Box<dyn internal::Measure<T>>,
                    Box<dyn internal::ComputeAggregation>,
                ),
            >,
            input: Vec<Vec<T>>,
            want: data::ExponentialHistogram<T>,
            want_count: usize,
        }
        let test_cases: Vec<TestCase<T>> = vec![
            TestCase {
                name: "Delta Single",
                build: Box::new(move || {
                    box_val(
                        AggregateBuilder::new(Some(Temporality::Delta), None)
                            .exponential_bucket_histogram(
                                max_size,
                                max_scale,
                                record_min_max,
                                record_sum,
                            ),
                    )
                }),
                input: vec![vec![4, 4, 4, 2, 16, 1]
                    .into_iter()
                    .map(Into::into)
                    .collect()],
                want: data::ExponentialHistogram {
                    temporality: Temporality::Delta,
                    data_points: vec![data::ExponentialHistogramDataPoint {
                        attributes: AttributeSet::default(),
                        count: 6,
                        min: Some(1.into()),
                        max: Some(16.into()),
                        sum: 31.into(),
                        start_time: SystemTime::now(),
                        time: SystemTime::now(),
                        scale: -1,
                        positive_bucket: data::ExponentialBucket {
                            offset: -1,
                            counts: vec![1, 4, 1],
                        },
                        negative_bucket: data::ExponentialBucket {
                            offset: 0,
                            counts: vec![],
                        },
                        exemplars: vec![],
                        zero_threshold: 0.0,
                        zero_count: 0,
                    }],
                },
                want_count: 1,
            },
            TestCase {
                name: "Cumulative Single",
                build: Box::new(move || {
                    box_val(
                        internal::AggregateBuilder::new(Some(Temporality::Cumulative), None)
                            .exponential_bucket_histogram(
                                max_size,
                                max_scale,
                                record_min_max,
                                record_sum,
                            ),
                    )
                }),
                input: vec![vec![4, 4, 4, 2, 16, 1]
                    .into_iter()
                    .map(Into::into)
                    .collect()],
                want: data::ExponentialHistogram {
                    temporality: Temporality::Cumulative,
                    data_points: vec![data::ExponentialHistogramDataPoint {
                        attributes: AttributeSet::default(),
                        count: 6,
                        min: Some(1.into()),
                        max: Some(16.into()),
                        sum: 31.into(),
                        scale: -1,
                        positive_bucket: data::ExponentialBucket {
                            offset: -1,
                            counts: vec![1, 4, 1],
                        },
                        start_time: SystemTime::now(),
                        time: SystemTime::now(),
                        negative_bucket: data::ExponentialBucket {
                            offset: 0,
                            counts: vec![],
                        },
                        exemplars: vec![],
                        zero_threshold: 0.0,
                        zero_count: 0,
                    }],
                },
                want_count: 1,
            },
            TestCase {
                name: "Delta Multiple",
                build: Box::new(move || {
                    box_val(
                        internal::AggregateBuilder::new(Some(Temporality::Delta), None)
                            .exponential_bucket_histogram(
                                max_size,
                                max_scale,
                                record_min_max,
                                record_sum,
                            ),
                    )
                }),
                input: vec![
                    vec![2, 3, 8].into_iter().map(Into::into).collect(),
                    vec![4, 4, 4, 2, 16, 1]
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                ],
                want: data::ExponentialHistogram {
                    temporality: Temporality::Delta,
                    data_points: vec![data::ExponentialHistogramDataPoint {
                        attributes: AttributeSet::default(),
                        count: 6,
                        min: Some(1.into()),
                        max: Some(16.into()),
                        sum: 31.into(),
                        scale: -1,
                        positive_bucket: data::ExponentialBucket {
                            offset: -1,
                            counts: vec![1, 4, 1],
                        },
                        start_time: SystemTime::now(),
                        time: SystemTime::now(),
                        negative_bucket: data::ExponentialBucket {
                            offset: 0,
                            counts: vec![],
                        },
                        exemplars: vec![],
                        zero_threshold: 0.0,
                        zero_count: 0,
                    }],
                },
                want_count: 1,
            },
            TestCase {
                name: "Cumulative Multiple ",
                build: Box::new(move || {
                    box_val(
                        internal::AggregateBuilder::new(Some(Temporality::Cumulative), None)
                            .exponential_bucket_histogram(
                                max_size,
                                max_scale,
                                record_min_max,
                                record_sum,
                            ),
                    )
                }),
                input: vec![
                    vec![2, 3, 8].into_iter().map(Into::into).collect(),
                    vec![4, 4, 4, 2, 16, 1]
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                ],
                want: data::ExponentialHistogram {
                    temporality: Temporality::Cumulative,
                    data_points: vec![data::ExponentialHistogramDataPoint {
                        count: 9,
                        min: Some(1.into()),
                        max: Some(16.into()),
                        sum: 44.into(),
                        scale: -1,
                        positive_bucket: data::ExponentialBucket {
                            offset: -1,
                            counts: vec![1, 6, 2],
                        },
                        attributes: AttributeSet::default(),
                        start_time: SystemTime::now(),
                        time: SystemTime::now(),
                        negative_bucket: data::ExponentialBucket {
                            offset: 0,
                            counts: vec![],
                        },
                        exemplars: vec![],
                        zero_threshold: 0.0,
                        zero_count: 0,
                    }],
                },
                want_count: 1,
            },
        ];

        for test in test_cases {
            let (in_fn, out_fn) = (test.build)();

            let mut got: Box<dyn data::Aggregation> = Box::new(data::ExponentialHistogram::<T> {
                data_points: vec![],
                temporality: Temporality::Delta,
            });
            let mut count = 0;
            for n in test.input {
                for v in n {
                    in_fn.call(v, AttributeSet::default())
                }
                count = out_fn.call(Some(got.as_mut())).0
            }

            assert_aggregation_eq::<T>(Box::new(test.want), got, true, test.name);
            assert_eq!(test.want_count, count, "{}", test.name);
        }
    }

    fn assert_aggregation_eq<T: Number<T> + PartialEq>(
        a: Box<dyn Aggregation>,
        b: Box<dyn Aggregation>,
        ignore_timestamp: bool,
        test_name: &'static str,
    ) {
        assert_eq!(
            a.as_any().type_id(),
            b.as_any().type_id(),
            "{} Aggregation types not equal",
            test_name
        );

        if let Some(a) = a.as_any().downcast_ref::<data::Gauge<T>>() {
            let b = b.as_any().downcast_ref::<data::Gauge<T>>().unwrap();
            assert_eq!(
                a.data_points.len(),
                b.data_points.len(),
                "{} gauge counts",
                test_name
            );
            for (a, b) in a.data_points.iter().zip(b.data_points.iter()) {
                assert_data_points_eq(
                    a,
                    b,
                    ignore_timestamp,
                    "mismatching gauge data points",
                    test_name,
                );
            }
        } else if let Some(a) = a.as_any().downcast_ref::<data::Sum<T>>() {
            let b = b.as_any().downcast_ref::<data::Sum<T>>().unwrap();
            assert_eq!(
                a.temporality, b.temporality,
                "{} mismatching sum temporality",
                test_name
            );
            assert_eq!(
                a.is_monotonic, b.is_monotonic,
                "{} mismatching sum monotonicity",
                test_name,
            );
            assert_eq!(
                a.data_points.len(),
                b.data_points.len(),
                "{} sum counts",
                test_name
            );
            for (a, b) in a.data_points.iter().zip(b.data_points.iter()) {
                assert_data_points_eq(
                    a,
                    b,
                    ignore_timestamp,
                    "mismatching sum data points",
                    test_name,
                );
            }
        } else if let Some(a) = a.as_any().downcast_ref::<data::Histogram<T>>() {
            let b = b.as_any().downcast_ref::<data::Histogram<T>>().unwrap();
            assert_eq!(
                a.temporality, b.temporality,
                "{}: mismatching hist temporality",
                test_name
            );
            assert_eq!(
                a.data_points.len(),
                b.data_points.len(),
                "{} hist counts",
                test_name
            );
            for (a, b) in a.data_points.iter().zip(b.data_points.iter()) {
                assert_hist_data_points_eq(
                    a,
                    b,
                    ignore_timestamp,
                    "mismatching hist data points",
                    test_name,
                );
            }
        } else if let Some(a) = a.as_any().downcast_ref::<data::ExponentialHistogram<T>>() {
            let b = b
                .as_any()
                .downcast_ref::<data::ExponentialHistogram<T>>()
                .unwrap();
            assert_eq!(
                a.temporality, b.temporality,
                "{} mismatching hist temporality",
                test_name
            );
            assert_eq!(
                a.data_points.len(),
                b.data_points.len(),
                "{} hist counts",
                test_name
            );
            for (a, b) in a.data_points.iter().zip(b.data_points.iter()) {
                assert_exponential_hist_data_points_eq(
                    a,
                    b,
                    ignore_timestamp,
                    "mismatching hist data points",
                    test_name,
                );
            }
        } else {
            panic!("Aggregation of unknown types")
        }
    }

    fn assert_data_points_eq<T: Number<T>>(
        a: &data::DataPoint<T>,
        b: &data::DataPoint<T>,
        ignore_timestamp: bool,
        message: &'static str,
        test_name: &'static str,
    ) {
        assert_eq!(
            a.attributes, b.attributes,
            "{}: {} attributes",
            test_name, message
        );
        assert_eq!(a.value, b.value, "{}: {} value", test_name, message);

        if !ignore_timestamp {
            assert_eq!(
                a.start_time, b.start_time,
                "{}: {} start time",
                test_name, message
            );
            assert_eq!(a.time, b.time, "{}: {} time", test_name, message);
        }
    }

    fn assert_hist_data_points_eq<T: Number<T>>(
        a: &data::HistogramDataPoint<T>,
        b: &data::HistogramDataPoint<T>,
        ignore_timestamp: bool,
        message: &'static str,
        test_name: &'static str,
    ) {
        assert_eq!(
            a.attributes, b.attributes,
            "{}: {} attributes",
            test_name, message
        );
        assert_eq!(a.count, b.count, "{}: {} count", test_name, message);
        assert_eq!(a.bounds, b.bounds, "{}: {} bounds", test_name, message);
        assert_eq!(
            a.bucket_counts, b.bucket_counts,
            "{}: {} bucket counts",
            test_name, message
        );
        assert_eq!(a.min, b.min, "{}: {} min", test_name, message);
        assert_eq!(a.max, b.max, "{}: {} max", test_name, message);
        assert_eq!(a.sum, b.sum, "{}: {} sum", test_name, message);

        if !ignore_timestamp {
            assert_eq!(
                a.start_time, b.start_time,
                "{}: {} start time",
                test_name, message
            );
            assert_eq!(a.time, b.time, "{}: {} time", test_name, message);
        }
    }

    fn assert_exponential_hist_data_points_eq<T: Number<T>>(
        a: &data::ExponentialHistogramDataPoint<T>,
        b: &data::ExponentialHistogramDataPoint<T>,
        ignore_timestamp: bool,
        message: &'static str,
        test_name: &'static str,
    ) {
        assert_eq!(
            a.attributes, b.attributes,
            "{}: {} attributes",
            test_name, message
        );
        assert_eq!(a.count, b.count, "{}: {} count", test_name, message);
        assert_eq!(a.min, b.min, "{}: {} min", test_name, message);
        assert_eq!(a.max, b.max, "{}: {} max", test_name, message);
        assert_eq!(a.sum, b.sum, "{}: {} sum", test_name, message);

        assert_eq!(a.scale, b.scale, "{}: {} scale", test_name, message);
        assert_eq!(
            a.zero_count, b.zero_count,
            "{}: {} zeros",
            test_name, message
        );

        assert_eq!(
            a.positive_bucket, b.positive_bucket,
            "{}: {} pos",
            test_name, message
        );
        assert_eq!(
            a.negative_bucket, b.negative_bucket,
            "{}: {} neg",
            test_name, message
        );

        if !ignore_timestamp {
            assert_eq!(
                a.start_time, b.start_time,
                "{}: {} start time",
                test_name, message
            );
            assert_eq!(a.time, b.time, "{}: {} time", test_name, message);
        }
    }
}
