/// Determines which measurements are eligible to become [exemplars].
///
/// An exemplar is a sample measurement retained alongside an aggregated data
/// point, carrying the trace and span id that were active when the measurement
/// was recorded. This lets a backend link from a bucket in a histogram straight
/// to a representative trace.
///
/// [exemplars]: crate::metrics::data::Exemplar
///
/// # Example
///
/// ```
/// use opentelemetry_sdk::metrics::{ExemplarFilter, SdkMeterProvider};
///
/// let provider = SdkMeterProvider::builder()
///     .with_exemplar_filter(ExemplarFilter::AlwaysOn)
///     .build();
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExemplarFilter {
    /// Every measurement is eligible to become an exemplar.
    ///
    /// Measurements recorded outside a sampled span still produce an exemplar,
    /// with an all-zero trace id and span id.
    AlwaysOn,

    /// No measurement is eligible. Exemplar collection is effectively disabled
    /// and costs nothing on the measurement path.
    AlwaysOff,

    /// Only measurements recorded inside a **sampled** span are eligible.
    ///
    /// This is the default, per the OpenTelemetry metrics SDK specification.
    #[default]
    TraceBased,
}
