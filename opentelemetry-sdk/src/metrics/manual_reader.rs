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
    reader::{DefaultTemporalitySelector, MetricReader, SdkProducer, TemporalitySelector},
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

#[derive(Debug)]
struct ManualReaderInner {
    sdk_producer: Option<Weak<dyn SdkProducer>>,
    is_shutdown: bool,
}

impl ManualReader {
    /// Configuration for this reader
    pub fn builder() -> ManualReaderBuilder {
        ManualReaderBuilder::default()
    }

    /// A [MetricReader] which is directly called to collect metrics.
    pub(crate) fn new(temporality_selector: Box<dyn TemporalitySelector>) -> Self {
        ManualReader {
            inner: Box::new(Mutex::new(ManualReaderInner {
                sdk_producer: None,
                is_shutdown: false,
            })),
            temporality_selector,
        }
    }
}

impl TemporalitySelector for ManualReader {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.temporality_selector.temporality(kind)
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

    /// Gathers all metrics from the SDK, calling any
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

        Ok(())
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

        Ok(())
    }
}

/// Configuration for a [ManualReader]
pub struct ManualReaderBuilder {
    temporality_selector: Box<dyn TemporalitySelector>,
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

    /// Create a new [ManualReader] from this configuration.
    pub fn build(self) -> ManualReader {
        ManualReader::new(self.temporality_selector)
    }
}
