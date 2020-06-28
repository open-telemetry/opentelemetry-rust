use crate::api::{
    metrics::{Descriptor, MetricsError, Number, Result},
    Context,
};
use crate::sdk::export::metrics::{Aggregator, Subtractor, Sum};
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

impl Subtractor for SumAggregator {
    fn subtract(
        &self,
        operand: &(dyn Aggregator + Send + Sync),
        result: &(dyn Aggregator + Send + Sync),
        descriptor: &Descriptor,
    ) -> Result<()> {
        match (
            operand.as_any().downcast_ref::<Self>(),
            result.as_any().downcast_ref::<Self>(),
        ) {
            (Some(op), Some(res)) => {
                res.value.assign(descriptor.number_kind(), &self.value);
                res.value
                    .saturating_sub(descriptor.number_kind(), &op.value);
                Ok(())
            }
            _ => Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?} and {:?}",
                self, operand, result
            ))),
        }
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
    fn synchronized_move(
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
