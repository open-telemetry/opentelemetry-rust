use crate::api::{
    metrics::{Descriptor, MetricsError, Number, NumberKind, Result},
    Context,
};
use crate::sdk::export::metrics::{Aggregator, Count, Max, Min, MinMaxSumCount, Sum};
use std::any::Any;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

/// Create a new `MinMaxSumCountAggregator`
pub fn min_max_sum_count(descriptor: &Descriptor) -> MinMaxSumCountAggregator {
    let kind = descriptor.number_kind().clone();
    MinMaxSumCountAggregator {
        inner: Mutex::new(Inner { state: None }),
        kind,
    }
}

#[derive(Debug)]
struct Inner {
    state: Option<State>,
}

/// An `Aggregator` that aggregates events that form a distribution, keeping
/// only the min, max, sum, and count.
#[derive(Debug)]
pub struct MinMaxSumCountAggregator {
    inner: Mutex<Inner>,
    kind: NumberKind,
}

impl Min for MinMaxSumCountAggregator {
    fn min(&self) -> Result<Number> {
        self.inner.lock().map_err(From::from).map(|inner| {
            inner
                .state
                .as_ref()
                .map_or(0u64.into(), |state| state.min.clone())
        })
    }
}

impl Max for MinMaxSumCountAggregator {
    fn max(&self) -> Result<Number> {
        self.inner.lock().map_err(From::from).map(|inner| {
            inner
                .state
                .as_ref()
                .map_or(0u64.into(), |state| state.max.clone())
        })
    }
}

impl Sum for MinMaxSumCountAggregator {
    fn sum(&self) -> Result<Number> {
        self.inner.lock().map_err(From::from).map(|inner| {
            inner
                .state
                .as_ref()
                .map_or(0u64.into(), |state| state.sum.clone())
        })
    }
}

impl Count for MinMaxSumCountAggregator {
    fn count(&self) -> Result<u64> {
        self.inner
            .lock()
            .map_err(From::from)
            .map(|inner| inner.state.as_ref().map_or(0u64, |state| state.count))
    }
}

impl MinMaxSumCount for MinMaxSumCountAggregator {}

impl Aggregator for MinMaxSumCountAggregator {
    fn update_with_context(
        &self,
        _cx: &Context,
        number: &Number,
        descriptor: &Descriptor,
    ) -> Result<()> {
        self.inner
            .lock()
            .and_then(|mut inner| {
                if let Some(state) = &mut inner.state {
                    let kind = descriptor.number_kind();

                    state.count = state.count.saturating_add(1);
                    state.sum.saturating_add(kind, number);
                    if number.partial_cmp(kind, &state.min) == Some(Ordering::Less) {
                        state.min = number.clone();
                    }
                    if number.partial_cmp(kind, &state.max) == Some(Ordering::Greater) {
                        state.max = number.clone();
                    }
                } else {
                    inner.state = Some(State {
                        count: 1,
                        sum: number.clone(),
                        min: number.clone(),
                        max: number.clone(),
                    })
                }

                Ok(())
            })
            .map_err(From::from)
    }

    fn synchronized_move(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        _descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(From::from).and_then(|mut inner| {
                other.inner.lock().map_err(From::from).map(|mut oi| {
                    oi.state = inner.state.take();
                })
            })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn merge(&self, aggregator: &(dyn Aggregator + Send + Sync), desc: &Descriptor) -> Result<()> {
        if let Some(other) = aggregator.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(From::from).and_then(|mut inner| {
                other.inner.lock().map_err(From::from).and_then(|oi| {
                    match (inner.state.as_mut(), oi.state.as_ref()) {
                        (None, Some(other_checkpoint)) => {
                            inner.state = Some(other_checkpoint.clone());
                        }
                        (Some(_), None) | (None, None) => (),
                        (Some(state), Some(other)) => {
                            state.count = state.count.saturating_add(other.count);
                            state.sum.saturating_add(desc.number_kind(), &other.sum);

                            if state.min.partial_cmp(desc.number_kind(), &other.min)
                                == Some(Ordering::Greater)
                            {
                                state.min.assign(desc.number_kind(), &other.min);
                            }
                            if state.max.partial_cmp(desc.number_kind(), &other.max)
                                == Some(Ordering::Less)
                            {
                                state.max.assign(desc.number_kind(), &other.max);
                            }
                        }
                    }
                    Ok(())
                })
            })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, aggregator
            )))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
struct State {
    count: u64,
    sum: Number,
    min: Number,
    max: Number,
}
