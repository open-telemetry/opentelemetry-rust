/// Determines which measurements are eligible to become [exemplars].
///
/// An exemplar is a sample measurement retained alongside an aggregated data
/// point, carrying the trace and span id that were active when the measurement
/// was recorded. This lets a backend link from a bucket in a histogram straight
/// to a representative trace.
///
/// [exemplars]: crate::metrics::data::Exemplar
///
/// # Attributes removed by a view are still exported
///
/// As the specification requires, an exemplar retains every attribute of its
/// measurement that the aggregated data point does not keep. An attribute
/// removed by a view's attribute filter therefore leaves the process on the
/// exemplar, with its original value. A view's attribute filter reduces
/// cardinality; it is **not** a redaction mechanism once exemplars are
/// collected. If a view is dropping an attribute because its value is
/// sensitive, do not record that attribute on the measurement, or use
/// [`ExemplarFilter::AlwaysOff`].
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
    /// Every measurement is eligible to become an exemplar, whether or not it
    /// was recorded inside a span, and whether or not that span was sampled.
    ///
    /// The exemplar carries the trace id and span id of the active span, even
    /// when that span was not sampled. A measurement recorded outside any span
    /// gets an all-zero trace id and span id.
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
