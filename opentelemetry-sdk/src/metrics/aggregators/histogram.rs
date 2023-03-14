use crate::export::metrics::aggregation::{
    Aggregation, AggregationKind, Buckets, Count, Histogram, Max, Min, Sum,
};
use crate::metrics::{
    aggregators::Aggregator,
    sdk_api::{AtomicNumber, Descriptor, Number, NumberKind},
};
use opentelemetry_api::metrics::{MetricsError, Result};
use opentelemetry_api::Context;
use std::mem;
use std::sync::{Arc, RwLock};

/// Create a new histogram for the given descriptor with the given boundaries
pub fn histogram(boundaries: &[f64]) -> HistogramAggregator {
    let mut sorted_boundaries = boundaries.to_owned();
    sorted_boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let state = State::empty(&sorted_boundaries);

    HistogramAggregator {
        inner: RwLock::new(Inner {
            boundaries: sorted_boundaries,
            state,
        }),
    }
}

/// This aggregator observes events and counts them in pre-determined buckets. It
/// also calculates the sum and count of all events.
#[derive(Debug)]
pub struct HistogramAggregator {
    inner: RwLock<Inner>,
}

#[derive(Debug)]
struct Inner {
    boundaries: Vec<f64>,
    state: State,
}

#[derive(Debug)]
struct State {
    bucket_counts: Vec<f64>,
    count: AtomicNumber,
    sum: AtomicNumber,
    min: AtomicNumber,
    max: AtomicNumber,
}

impl State {
    fn empty(boundaries: &[f64]) -> Self {
        State {
            bucket_counts: vec![0.0; boundaries.len() + 1],
            count: NumberKind::U64.zero().to_atomic(),
            sum: NumberKind::U64.zero().to_atomic(),
            min: NumberKind::I64.max().to_atomic(), // TODO: This could result in some fun bugs,
            // but we currently don't have access to the
            // kind so this could be a problem.
            max: NumberKind::U64.min().to_atomic(),
        }
    }
}

impl Sum for HistogramAggregator {
    fn sum(&self) -> Result<Number> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.state.sum.load())
    }
}

impl Min for HistogramAggregator {
    fn min(&self) -> Result<Number> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.state.min.load())
    }
}

impl Max for HistogramAggregator {
    fn max(&self) -> Result<Number> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.state.max.load())
    }
}

impl Count for HistogramAggregator {
    fn count(&self) -> Result<u64> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.state.count.load().to_u64(&NumberKind::U64))
    }
}

impl Histogram for HistogramAggregator {
    fn histogram(&self) -> Result<Buckets> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| Buckets::new(inner.boundaries.clone(), inner.state.bucket_counts.clone()))
    }
}

impl Aggregation for HistogramAggregator {
    fn kind(&self) -> &AggregationKind {
        &AggregationKind::HISTOGRAM
    }
}

impl Aggregator for HistogramAggregator {
    fn aggregation(&self) -> &dyn Aggregation {
        self
    }
    fn update(&self, _cx: &Context, number: &Number, descriptor: &Descriptor) -> Result<()> {
        self.inner.write().map_err(From::from).map(|mut inner| {
            let kind = descriptor.number_kind();
            let as_float = number.to_f64(kind);

            let mut bucket_id = inner.boundaries.len();
            for (idx, boundary) in inner.boundaries.iter().enumerate() {
                if as_float < *boundary {
                    bucket_id = idx;
                    break;
                }
            }

            inner.state.count.fetch_add(&NumberKind::U64, &1u64.into());
            inner.state.sum.fetch_add(kind, number);
            inner.state.min.fetch_set_min(kind, number);
            inner.state.max.fetch_set_max(kind, number);
            inner.state.bucket_counts[bucket_id] += 1.0;
        })
    }

    fn synchronized_move(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        _descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.inner
                .write()
                .map_err(From::from)
                .and_then(|mut inner| {
                    other.inner.write().map_err(From::from).map(|mut other| {
                        let empty = State::empty(&inner.boundaries);
                        other.state = mem::replace(&mut inner.state, empty)
                    })
                })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn merge(&self, other: &(dyn Aggregator + Send + Sync), desc: &Descriptor) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<HistogramAggregator>() {
            self.inner
                .write()
                .map_err(From::from)
                .and_then(|mut inner| {
                    other.inner.read().map_err(From::from).map(|other| {
                        inner
                            .state
                            .sum
                            .fetch_add(desc.number_kind(), &other.state.sum.load());
                        inner
                            .state
                            .count
                            .fetch_add(&NumberKind::U64, &other.state.count.load());
                        inner
                            .state
                            .min
                            .fetch_set_min(desc.number_kind(), &other.state.min.load());
                        inner
                            .state
                            .max
                            .fetch_set_min(desc.number_kind(), &other.state.max.load());
                        for idx in 0..inner.state.bucket_counts.len() {
                            inner.state.bucket_counts[idx] += other.state.bucket_counts[idx];
                        }
                    })
                })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
