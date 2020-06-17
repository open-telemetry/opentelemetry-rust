use crate::api::{
    metrics::{Descriptor, MetricsError, Number, Result},
    Context,
};
use crate::sdk::export::metrics::{Aggregator, Sum};
use std::any::Any;
use std::sync::Arc;

/// Create a new sum aggregator.
pub fn sum() -> SumAggregator {
    SumAggregator::default()
}

/// An aggregator for counter events.
#[derive(Debug, Default)]
pub struct SumAggregator {
    value: Number,
}

impl Sum for SumAggregator {
    fn sum(&self) -> Result<Number> {
        Ok(self.value.clone())
    }
}

impl Aggregator for SumAggregator {
    fn update_with_context(
        &self,
        _cx: &Context,
        number: &Number,
        descriptor: &Descriptor,
    ) -> Result<()> {
        self.value.saturating_add(descriptor.number_kind(), number);
        Ok(())
    }
    fn synchronized_copy(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let kind = descriptor.number_kind();
            other.value.assign(kind, &self.value);
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
                .saturating_add(descriptor.number_kind(), &other_sum.value)
        }

        Ok(())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
