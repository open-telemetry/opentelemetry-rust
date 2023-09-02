use std::{
    fmt,
    sync::{Mutex, Weak},
};

use opentelemetry::{
    global,
    metrics::{MetricsError, Result},
};

use super::{
    data::{ResourceMetrics, Temporality},
    instrument::InstrumentKind,
    pipeline::Pipeline,
    reader::{
        AggregationSelector, DefaultAggregationSelector, DefaultTemporalitySelector,
        MetricProducer, MetricReader, SdkProducer, TemporalitySelector,
    },
};

/// A simple [MetricReader] that allows an application to read metrics on demand.
///
/// See [ManualReaderBuilder] for configuration options.
///
/// # Example
///
/// ```
/// use opentelemetry_sdk::metrics::ManualReader;
///
/// // can specify additional reader configuration
/// let reader = ManualReader::builder().build();
/// # drop(reader)
/// ```
pub struct ManualReader {
    inner: Box<Mutex<ManualReaderInner>>,
    temporality_selector: Box<dyn TemporalitySelector>,
    aggregation_selector: Box<dyn AggregationSelector>,
}

impl Default for ManualReader {
    fn default() -> Self {
        ManualReader::builder().build()
    }
}

impl fmt::Debug for ManualReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ManualReader")
    }
}

#[derive(Debug, Default)]
struct ManualReaderInner {
    sdk_producer: Option<Weak<dyn SdkProducer>>,
    is_shutdown: bool,
    external_producers: Vec<Box<dyn MetricProducer>>,
}

impl ManualReader {
    /// Configuration for this reader
    pub fn builder() -> ManualReaderBuilder {
        ManualReaderBuilder::default()
    }

    /// A [MetricReader] which is directly called to collect metrics.
    pub(crate) fn new(
        temporality_selector: Box<dyn TemporalitySelector>,
        aggregation_selector: Box<dyn AggregationSelector>,
    ) -> Self {
        ManualReader {
            inner: Box::new(Mutex::new(ManualReaderInner::default())),
            temporality_selector,
            aggregation_selector,
        }
    }
}

impl TemporalitySelector for ManualReader {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.temporality_selector.temporality(kind)
    }
}

impl AggregationSelector for ManualReader {
    fn aggregation(&self, kind: InstrumentKind) -> super::aggregation::Aggregation {
        self.aggregation_selector.aggregation(kind)
    }
}

impl MetricReader for ManualReader {
    ///  Register a pipeline which enables the caller to read metrics from the SDK
    ///  on demand.
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        let _ = self.inner.lock().map(|mut inner| {
            // Only register once. If producer is already set, do nothing.
            if inner.sdk_producer.is_none() {
                inner.sdk_producer = Some(pipeline);
            } else {
                global::handle_error(MetricsError::Config(
                    "duplicate reader registration, did not register manual reader".into(),
                ))
            }
        });
    }

    /// Stores the external [MetricProducer] which enables the caller to read
    /// metrics on demand.
    fn register_producer(&self, producer: Box<dyn MetricProducer>) {
        let _ = self.inner.lock().map(|mut inner| {
            if !inner.is_shutdown {
                inner.external_producers.push(producer);
            }
        });
    }

    /// Gathers all metrics from the SDK and other [MetricProducer]s, calling any
    /// callbacks necessary and returning the results.
    ///
    /// Returns an error if called after shutdown.
    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()> {
        let inner = self.inner.lock()?;
        match &inner.sdk_producer.as_ref().and_then(|w| w.upgrade()) {
            Some(producer) => producer.produce(rm)?,
            None => {
                return Err(MetricsError::Other(
                    "reader is shut down or not registered".into(),
                ))
            }
        };

        let mut errs = vec![];
        for producer in &inner.external_producers {
            match producer.produce() {
                Ok(metrics) => rm.scope_metrics.push(metrics),
                Err(err) => errs.push(err),
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(MetricsError::Other(format!("{:?}", errs)))
        }
    }

    /// ForceFlush is a no-op, it always returns nil.
    fn force_flush(&self) -> Result<()> {
        Ok(())
    }

    /// Closes any connections and frees any resources used by the reader.
    fn shutdown(&self) -> Result<()> {
        let mut inner = self.inner.lock()?;

        // Any future call to collect will now return an error.
        inner.sdk_producer = None;
        inner.is_shutdown = true;
        inner.external_producers = Vec::new();

        Ok(())
    }
}

/// Configuration for a [ManualReader]
pub struct ManualReaderBuilder {
    temporality_selector: Box<dyn TemporalitySelector>,
    aggregation_selector: Box<dyn AggregationSelector>,
}

impl fmt::Debug for ManualReaderBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ManualReaderBuilder")
    }
}

impl Default for ManualReaderBuilder {
    fn default() -> Self {
        ManualReaderBuilder {
            temporality_selector: Box::new(DefaultTemporalitySelector { _private: () }),
            aggregation_selector: Box::new(DefaultAggregationSelector { _private: () }),
        }
    }
}

impl ManualReaderBuilder {
    /// New manual builder configuration
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the [TemporalitySelector] a reader will use to determine the [Temporality] of
    /// an instrument based on its kind. If this option is not used, the reader will use
    /// the default temporality selector.
    pub fn with_temporality_selector(
        mut self,
        temporality_selector: impl TemporalitySelector + 'static,
    ) -> Self {
        self.temporality_selector = Box::new(temporality_selector);
        self
    }

    /// Sets the [AggregationSelector] a reader will use to determine the
    /// aggregation to use for an instrument based on its kind.
    ///
    /// If this option is not used, the reader will use the default aggregation
    /// selector or the aggregation explicitly passed for a view matching an
    /// instrument.
    pub fn with_aggregation_selector(
        mut self,
        aggregation_selector: Box<dyn AggregationSelector>,
    ) -> Self {
        self.aggregation_selector = aggregation_selector;
        self
    }

    /// Create a new [ManualReader] from this configuration.
    pub fn build(self) -> ManualReader {
        ManualReader::new(self.temporality_selector, self.aggregation_selector)
    }
}
