//! Exemplar sampling for the metrics pipeline.
//!
//! Two shapes of every type here are compiled: a working one behind
//! `spec_unstable_metrics_exemplars`, and a zero-sized stand-in without it.
//! Both expose the same API, so the aggregators can hold a reservoir and offer
//! measurements to it unconditionally — with the feature off every call is a
//! no-op on a ZST and the optimizer removes it, keeping the measurement path
//! byte-identical to a build that never knew about exemplars.

#[cfg(feature = "spec_unstable_metrics_exemplars")]
mod enabled;
#[cfg(feature = "spec_unstable_metrics_exemplars")]
pub(crate) use enabled::*;

#[cfg(not(feature = "spec_unstable_metrics_exemplars"))]
mod disabled;
#[cfg(not(feature = "spec_unstable_metrics_exemplars"))]
pub(crate) use disabled::*;

use opentelemetry::KeyValue;

use super::aggregate::AttributeSetFilter;

/// The attributes a view's attribute filter removes from a measurement, which
/// the spec requires an exemplar to retain.
///
/// Borrowed and lazy: nothing is resolved or cloned unless a reservoir decides
/// to keep the measurement.
#[derive(Clone, Copy)]
#[cfg_attr(not(feature = "spec_unstable_metrics_exemplars"), allow(dead_code))]
pub(crate) enum DroppedAttributes<'a> {
    /// The measurement's full attribute set and the filter about to be applied
    /// to it. Used by unbound recordings, which see the attributes per call.
    Unresolved {
        attrs: &'a [KeyValue],
        filter: &'a AttributeSetFilter,
    },
    /// Attributes already known to be dropped. Used by bound instruments, which
    /// resolve them once at bind time because no attributes are passed when
    /// recording.
    #[cfg_attr(
        not(feature = "experimental_metrics_bound_instruments"),
        allow(dead_code)
    )]
    Resolved(&'a [KeyValue]),
}

#[cfg(feature = "spec_unstable_metrics_exemplars")]
impl DroppedAttributes<'_> {
    fn to_vec(self) -> Vec<KeyValue> {
        match self {
            DroppedAttributes::Unresolved { attrs, filter } => filter.dropped(attrs),
            DroppedAttributes::Resolved(dropped) => dropped.to_vec(),
        }
    }
}

impl std::fmt::Debug for DroppedAttributes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DroppedAttributes")
    }
}
