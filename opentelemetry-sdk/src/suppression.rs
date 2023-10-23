
use opentelemetry::Context;
use std::any::{Any, TypeId};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone, Copy)]
struct SuppressionKey(bool); // true means logging is suppressed

pub struct Suppression;

impl Suppression {
    pub fn new() -> Self {
        // Suppress logging when a new Suppression instance is created
        let mut new_context = Context::current();
        new_context.entries.insert(TypeId::of::<SuppressionKey>(), Arc::new(SuppressionKey(true)));
        new_context.attach();

        Suppression
    }

    pub fn is_logging_suppressed() -> bool {
        Context::current().get::<SuppressionKey>().cloned().unwrap_or(SuppressionKey(false)).0
    }
}

impl Drop for Suppression {
    fn drop(&mut self) {
        // Resume logging when the Suppression instance is dropped
        let mut new_context = Context::current();
        new_context.entries.insert(TypeId::of::<SuppressionKey>(), Arc::new(SuppressionKey(false)));
        new_context.attach();
    }
}