use crate::export::metrics::aggregation::{Aggregation, AggregationKind, Sum};
use crate::metrics::{
    aggregators::Aggregator,
    sdk_api::{AtomicNumber, Descriptor, Number},
};
use opentelemetry_api::metrics::{MetricsError, Result};
use opentelemetry_api::Context;
use std::any::Any;
use std::sync::Arc;

/// Create a new sum aggregator.
pub fn sum() -> impl Aggregator {
    SumAggregator::default()
}

/// An aggregator for counter events.
#[derive(Debug, Default)]
pub struct SumAggregator {
    value: AtomicNumber,
}

impl Sum for SumAggregator {
    fn sum(&self) -> Result<Number> {
        Ok(self.value.load())
    }
}

impl Aggregation for SumAggregator {
    fn kind(&self) -> &AggregationKind {
        &AggregationKind::SUM
    }
}

impl Aggregator for SumAggregator {
    fn aggregation(&self) -> &dyn Aggregation {
        self
    }

    fn update(&self, _cx: &Context, number: &Number, descriptor: &Descriptor) -> Result<()> {
        self.value.fetch_add(descriptor.number_kind(), number);
        Ok(())
    }

    fn synchronized_move(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let kind = descriptor.number_kind();
            other.value.store(&self.value.load());
            self.value.store(&kind.zero());
            Ok(())
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn merge(&self, other: &(dyn Aggregator + Send + Sync), descriptor: &Descriptor) -> Result<()> {
        if let Some(other_sum) = other.as_any().downcast_ref::<SumAggregator>() {
            self.value
                .fetch_add(descriptor.number_kind(), &other_sum.value.load())
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
