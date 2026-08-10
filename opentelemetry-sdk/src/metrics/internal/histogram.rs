use std::mem::replace;
use std::ops::DerefMut;
#[cfg(feature = "experimental_metrics_bound_instruments")]
use std::sync::atomic::Ordering;
#[cfg(feature = "experimental_metrics_bound_instruments")]
use std::sync::Arc;
use std::sync::Mutex;

use crate::metrics::data::{self, MetricData};
use crate::metrics::data::{AggregatedMetrics, HistogramDataPoint};
use crate::metrics::Temporality;
use opentelemetry::KeyValue;

use super::aggregate::{AggregateTimeInitiator, AttributeSetFilter};
use super::{
    Aggregator, AlignedHistogramBucketReservoir, ComputeAggregation, ExemplarOffer,
    ExemplarSampler, Measure, Number, ValueMap,
};
#[cfg(feature = "experimental_metrics_bound_instruments")]
use super::{BoundMeasure, NoopBoundMeasure, TrackerEntry};

impl<T> Aggregator for Mutex<Buckets<T>>
where
    T: Number,
{
    type InitConfig = usize;
    /// Value, bucket index, and — when the measurement is exemplar-eligible —
    /// the sampled trace context captured for it.
    type PreComputedValue = (T, usize, Option<Box<ExemplarOffer>>);

    fn update(&self, (value, index, exemplar): (T, usize, Option<Box<ExemplarOffer>>)) {
        let mut buckets = self.lock().unwrap_or_else(|err| err.into_inner());

        if let Some(offer) = exemplar {
            // Free of extra synchronization: this aggregator already holds the
            // lock for the counter update, and the bucket index the aligned
            // reservoir keys on was resolved during precomputation.
            buckets.exemplars.offer(value, index, *offer);
        }

        buckets.total += value;
        buckets.count += 1;
        if !buckets.counts.is_empty() {
            buckets.counts[index] += 1;
        }

        if value < buckets.min {
            buckets.min = value;
        }
        if value > buckets.max {
            buckets.max = value
        }
    }

    fn create(count: &usize) -> Self {
        Mutex::new(Buckets::<T>::new(*count))
    }

    fn clone_and_reset(&self, count: &usize) -> Self {
        let mut current = self.lock().unwrap_or_else(|err| err.into_inner());
        Mutex::new(replace(current.deref_mut(), Buckets::new(*count)))
    }
}

struct Buckets<T> {
    counts: Vec<u64>,
    count: u64,
    total: T,
    min: T,
    max: T,
    exemplars: AlignedHistogramBucketReservoir<T>,
}

impl<T: Number> Buckets<T> {
    /// returns buckets with `n` bins.
    fn new(n: usize) -> Buckets<T> {
        Buckets {
            counts: vec![0; n],
            count: 0,
            total: T::default(),
            min: T::max(),
            max: T::min(),
            // An empty-boundary histogram deliberately exports no bucket
            // counts, but every measurement still belongs to the single
            // conceptual bucket at index zero.
            exemplars: AlignedHistogramBucketReservoir::new(n.max(1)),
        }
    }
}

/// Pre-bound histogram handle: writes go directly to a fixed `TrackerEntry`
/// without per-call attribute lookup. The `tracker` is either a dedicated entry
/// for the bound attribute set, or — if bind() hit the cardinality limit — the
/// shared overflow tracker.
#[cfg(feature = "experimental_metrics_bound_instruments")]
struct BoundHistogramHandle<T: Number> {
    tracker: Arc<TrackerEntry<Mutex<Buckets<T>>>>,
    bounds: Vec<f64>,
    exemplars: ExemplarSampler,
}

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T: Number> BoundMeasure<T> for BoundHistogramHandle<T> {
    fn call(&self, measurement: T) {
        let f = measurement.into_float();
        let index = self.bounds.partition_point(|&x| x < f);
        self.tracker
            .aggregator
            .update((measurement, index, self.exemplars.offer()));
        self.tracker.has_been_updated.store(true, Ordering::Release);
    }
}

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T: Number> Drop for BoundHistogramHandle<T> {
    fn drop(&mut self) {
        self.tracker.bound_count.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Summarizes a set of measurements as a histogram with explicitly defined
/// buckets.
pub(crate) struct Histogram<T: Number> {
    value_map: ValueMap<Mutex<Buckets<T>>>,
    init_time: AggregateTimeInitiator,
    temporality: Temporality,
    filter: AttributeSetFilter,
    bounds: Vec<f64>,
    record_min_max: bool,
    record_sum: bool,
    exemplars: ExemplarSampler,
}

impl<T: Number> Histogram<T> {
    pub(crate) fn new(
        temporality: Temporality,
        filter: AttributeSetFilter,
        bounds: Vec<f64>,
        record_min_max: bool,
        record_sum: bool,
        cardinality_limit: usize,
        exemplars: ExemplarSampler,
    ) -> Self {
        let buckets_count = if bounds.is_empty() {
            0
        } else {
            bounds.len() + 1
        };

        Histogram {
            value_map: ValueMap::new(buckets_count, cardinality_limit),
            init_time: AggregateTimeInitiator::default(),
            temporality,
            filter,
            bounds,
            record_min_max,
            record_sum,
            exemplars,
        }
    }

    fn delta(&self, dest: Option<&mut MetricData<T>>) -> (usize, Option<MetricData<T>>) {
        let time = self.init_time.delta();

        let h = dest.and_then(|d| {
            if let MetricData::Histogram(hist) = d {
                Some(hist)
            } else {
                None
            }
        });
        let mut new_agg = if h.is_none() {
            Some(data::Histogram {
                data_points: vec![],
                start_time: time.start,
                time: time.current,
                temporality: Temporality::Delta,
            })
        } else {
            None
        };
        let h = h.unwrap_or_else(|| new_agg.as_mut().expect("present if h is none"));
        h.temporality = Temporality::Delta;
        h.start_time = time.start;
        h.time = time.current;

        let buckets_count = *self.value_map.config();
        self.value_map
            .collect_and_reset(&mut h.data_points, |attributes, aggr| {
                let reset = aggr.clone_and_reset(&buckets_count);
                let mut b = reset.into_inner().unwrap_or_else(|err| err.into_inner());
                HistogramDataPoint {
                    attributes,
                    count: b.count,
                    bounds: self.bounds.clone(),
                    bucket_counts: b.counts,
                    sum: if self.record_sum {
                        b.total
                    } else {
                        T::default()
                    },
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
                    exemplars: b.exemplars.take(),
                }
            });

        (h.data_points.len(), new_agg.map(Into::into))
    }

    fn cumulative(&self, dest: Option<&mut MetricData<T>>) -> (usize, Option<MetricData<T>>) {
        let time = self.init_time.cumulative();
        let h = dest.and_then(|d| {
            if let MetricData::Histogram(hist) = d {
                Some(hist)
            } else {
                None
            }
        });
        let mut new_agg = if h.is_none() {
            Some(data::Histogram {
                data_points: vec![],
                start_time: time.start,
                time: time.current,
                temporality: Temporality::Cumulative,
            })
        } else {
            None
        };
        let h = h.unwrap_or_else(|| new_agg.as_mut().expect("present if h is none"));
        h.temporality = Temporality::Cumulative;
        h.start_time = time.start;
        h.time = time.current;

        self.value_map
            .collect_readonly(&mut h.data_points, |attributes, aggr| {
                let mut b = aggr.lock().unwrap_or_else(|err| err.into_inner());
                HistogramDataPoint {
                    attributes,
                    count: b.count,
                    bounds: self.bounds.clone(),
                    bucket_counts: b.counts.clone(),
                    sum: if self.record_sum {
                        b.total
                    } else {
                        T::default()
                    },
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
                    // Drained even in cumulative temporality: an exemplar
                    // describes the interval it was sampled in, so holding it
                    // across cycles would keep re-exporting a stale trace id.
                    exemplars: b.exemplars.take(),
                }
            });

        (h.data_points.len(), new_agg.map(Into::into))
    }
}

impl<T> Measure<T> for Histogram<T>
where
    T: Number,
{
    fn call(&self, measurement: T, attrs: &[KeyValue]) {
        let f = measurement.into_float();
        // This search will return an index in the range `[0, bounds.len()]`, where
        // it will return `bounds.len()` if value is greater than the last element
        // of `bounds`. This aligns with the buckets in that the length of buckets
        // is `bounds.len()+1`, with the last bucket representing:
        // `(bounds[bounds.len()-1], +∞)`.
        let index = self.bounds.partition_point(|&x| x < f);

        // Resolved before the attribute filter runs so that `AlwaysOff` (and
        // any build without the exemplar feature) returns `None` here and the
        // rest of this path is unchanged.
        let mut exemplar = self.exemplars.offer();

        self.filter.apply(attrs, |filtered| {
            if let Some(offer) = exemplar.as_mut() {
                offer.set_filtered_attributes(attrs, filtered);
            }
            self.value_map
                .measure((measurement, index, exemplar.take()), filtered);
        })
    }

    #[cfg(feature = "experimental_metrics_bound_instruments")]
    fn bind(&self, attrs: &[KeyValue]) -> Box<dyn BoundMeasure<T>> {
        let mut bound_attrs = Vec::new();
        self.filter.apply(attrs, |filtered| {
            bound_attrs = filtered.to_vec();
        });
        match self.value_map.bind(&bound_attrs) {
            Some(tracker) => Box::new(BoundHistogramHandle {
                tracker,
                bounds: self.bounds.clone(),
                exemplars: self.exemplars,
            }),
            // Trackers RwLock is poisoned — return a noop handle so writes
            // silently drop, mirroring `measure()`'s own poison handling.
            None => Box::new(NoopBoundMeasure::new()),
        }
    }
}

impl<T> ComputeAggregation for Histogram<T>
where
    T: Number,
{
    fn call(&self, dest: Option<&mut AggregatedMetrics>) -> (usize, Option<AggregatedMetrics>) {
        let data = dest.and_then(|d| T::extract_metrics_data_mut(d));
        let (len, new) = match self.temporality {
            Temporality::Delta => self.delta(data),
            _ => self.cumulative(data),
        };
        (len, new.map(T::make_aggregated_metrics))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_buckets_are_selected_correctly() {
        let hist = Histogram::<i64>::new(
            Temporality::Cumulative,
            AttributeSetFilter::new(None),
            vec![1.0, 3.0, 6.0],
            false,
            false,
            2000,
            ExemplarSampler::default(),
        );
        for v in 1..11 {
            Measure::call(&hist, v, &[]);
        }
        let (count, dp) = ComputeAggregation::call(&hist, None);
        let dp = dp.unwrap();
        let AggregatedMetrics::I64(MetricData::Histogram(dp)) = dp else {
            unreachable!()
        };
        assert_eq!(count, 1);
        assert_eq!(dp.data_points[0].count, 10);
        assert_eq!(dp.data_points[0].bucket_counts.len(), 4);
        assert_eq!(dp.data_points[0].bucket_counts[0], 1); // 1
        assert_eq!(dp.data_points[0].bucket_counts[1], 2); // 2, 3
        assert_eq!(dp.data_points[0].bucket_counts[2], 3); // 4, 5, 6
        assert_eq!(dp.data_points[0].bucket_counts[3], 4); // 7, 8, 9, 10
    }
}

#[cfg(all(test, feature = "spec_unstable_metrics_exemplars"))]
mod exemplar_tests {
    use opentelemetry::trace::{SpanContext, TraceContextExt, TraceState};
    use opentelemetry::{Context, ContextGuard, SpanId, TraceFlags, TraceId};

    use super::*;
    use crate::metrics::data::Exemplar;
    use crate::metrics::ExemplarFilter;

    const TRACE_ID: u128 = 0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10;
    const SPAN_ID: u64 = 0x1112_1314_1516_1718;

    fn hist_with_bounds(filter: ExemplarFilter, bounds: Vec<f64>) -> Histogram<i64> {
        Histogram::<i64>::new(
            Temporality::Delta,
            AttributeSetFilter::new(None),
            bounds,
            false,
            false,
            2000,
            ExemplarSampler::new(filter),
        )
    }

    fn hist(filter: ExemplarFilter) -> Histogram<i64> {
        hist_with_bounds(filter, vec![1.0, 3.0, 6.0])
    }

    /// Attaches a span context to the current thread for the guard's lifetime.
    fn active_span(flags: TraceFlags) -> ContextGuard {
        let span_cx = SpanContext::new(
            TraceId::from(TRACE_ID),
            SpanId::from(SPAN_ID),
            flags,
            false,
            TraceState::default(),
        );
        Context::current()
            .with_remote_span_context(span_cx)
            .attach()
    }

    fn collect(hist: &Histogram<i64>) -> Vec<Exemplar<i64>> {
        let (_, dp) = ComputeAggregation::call(hist, None);
        let Some(AggregatedMetrics::I64(MetricData::Histogram(h))) = dp else {
            unreachable!()
        };
        h.data_points
            .into_iter()
            .flat_map(|dp| dp.exemplars)
            .collect()
    }

    #[test]
    fn always_off_collects_nothing_even_inside_a_sampled_span() {
        let hist = hist(ExemplarFilter::AlwaysOff);
        let _guard = active_span(TraceFlags::SAMPLED);
        Measure::call(&hist, 2, &[]);

        assert!(collect(&hist).is_empty());
    }

    #[test]
    fn always_on_collects_outside_any_span() {
        let hist = hist(ExemplarFilter::AlwaysOn);
        Measure::call(&hist, 2, &[]);

        let exemplars = collect(&hist);
        assert_eq!(exemplars.len(), 1);
        assert_eq!(exemplars[0].value, 2);
        // No span was active, so the ids stay zeroed rather than the
        // measurement being dropped.
        assert_eq!(exemplars[0].trace_id, [0; 16]);
        assert_eq!(exemplars[0].span_id, [0; 8]);
    }

    #[test]
    fn always_on_captures_ids_of_an_active_unsampled_span() {
        let hist = hist(ExemplarFilter::AlwaysOn);
        {
            let _guard = active_span(TraceFlags::default());
            Measure::call(&hist, 2, &[]);
        }

        let exemplars = collect(&hist);
        assert_eq!(exemplars.len(), 1);
        assert_eq!(exemplars[0].trace_id, TraceId::from(TRACE_ID).to_bytes());
        assert_eq!(exemplars[0].span_id, SpanId::from(SPAN_ID).to_bytes());
    }

    #[test]
    fn empty_boundaries_still_retain_an_exemplar() {
        let hist = hist_with_bounds(ExemplarFilter::AlwaysOn, vec![]);
        Measure::call(&hist, 2, &[]);

        let exemplars = collect(&hist);
        assert_eq!(exemplars.len(), 1);
        assert_eq!(exemplars[0].value, 2);
    }

    #[test]
    fn trace_based_captures_ids_of_the_active_sampled_span() {
        let hist = hist(ExemplarFilter::TraceBased);
        {
            let _guard = active_span(TraceFlags::SAMPLED);
            Measure::call(&hist, 2, &[]);
        }

        let exemplars = collect(&hist);
        assert_eq!(exemplars.len(), 1);
        assert_eq!(exemplars[0].value, 2);
        assert_eq!(exemplars[0].trace_id, TraceId::from(TRACE_ID).to_bytes());
        assert_eq!(exemplars[0].span_id, SpanId::from(SPAN_ID).to_bytes());
    }

    #[test]
    fn trace_based_ignores_unsampled_spans_and_no_span_at_all() {
        let hist = hist(ExemplarFilter::TraceBased);
        {
            let _guard = active_span(TraceFlags::default());
            Measure::call(&hist, 2, &[]);
        }
        Measure::call(&hist, 4, &[]);

        assert!(collect(&hist).is_empty());
    }

    #[test]
    fn at_most_one_exemplar_is_kept_per_bucket() {
        let hist = hist(ExemplarFilter::AlwaysOn);
        // Bounds are [1, 3, 6], so four buckets; record ten values spread
        // across all of them.
        for v in 1..11 {
            Measure::call(&hist, v, &[]);
        }

        let exemplars = collect(&hist);
        assert_eq!(exemplars.len(), 4, "one exemplar per non-empty bucket");

        let bounds = [1.0, 3.0, 6.0];
        let mut buckets: Vec<usize> = exemplars
            .iter()
            .map(|e| bounds.partition_point(|&b| b < e.value as f64))
            .collect();
        buckets.sort_unstable();
        assert_eq!(buckets, vec![0, 1, 2, 3], "each bucket represented once");
    }

    #[test]
    fn exemplars_do_not_leak_across_collection_cycles() {
        let hist = hist(ExemplarFilter::AlwaysOn);
        Measure::call(&hist, 2, &[]);
        assert_eq!(collect(&hist).len(), 1);

        // Nothing recorded in this cycle, so the drained reservoir must not
        // re-export the previous cycle's exemplar.
        assert!(collect(&hist).is_empty());
    }

    #[test]
    fn cumulative_temporality_also_drains_each_cycle() {
        let hist = Histogram::<i64>::new(
            Temporality::Cumulative,
            AttributeSetFilter::new(None),
            vec![1.0, 3.0, 6.0],
            false,
            false,
            2000,
            ExemplarSampler::new(ExemplarFilter::AlwaysOn),
        );
        Measure::call(&hist, 2, &[]);
        assert_eq!(collect(&hist).len(), 1);
        assert!(collect(&hist).is_empty());
    }
}
