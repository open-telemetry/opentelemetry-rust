use opentelemetry::KeyValue;

/// A trait for processing measurement attributes.
pub trait MeasurementProcessor: Send + Sync + 'static {

    /// Processes the attributes of a measurement.
    ///
    /// The processor might decide to modify the attributes. In that case, it returns
    /// `Some` with the modified attributes. If no attribute modification is needed,
    /// it returns `None`.
    fn process<'a>(&self, attributes: &[KeyValue]) -> Option<Vec<KeyValue>>;
}

