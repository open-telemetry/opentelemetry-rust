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
