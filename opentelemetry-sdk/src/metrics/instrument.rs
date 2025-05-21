use std::{borrow::Cow, collections::HashSet, error::Error, sync::Arc};

use opentelemetry::{
    metrics::{AsyncInstrument, SyncInstrument},
    InstrumentationScope, Key, KeyValue,
};

use crate::metrics::{aggregation::Aggregation, internal::Measure};

use super::Temporality;

/// The identifier of a group of instruments that all perform the same function.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum InstrumentKind {
    /// Identifies a group of instruments that record increasing values synchronously
    /// with the code path they are measuring.
    Counter,
    /// A group of instruments that record increasing and decreasing values
    /// synchronously with the code path they are measuring.
    UpDownCounter,
    /// A group of instruments that record a distribution of values synchronously with
    /// the code path they are measuring.
    Histogram,
    /// A group of instruments that record increasing values in an asynchronous
    /// callback.
    ObservableCounter,
    /// A group of instruments that record increasing and decreasing values in an
    /// asynchronous callback.
    ObservableUpDownCounter,

    /// a group of instruments that record current value synchronously with
    /// the code path they are measuring.
    Gauge,
    ///
    /// a group of instruments that record current values in an asynchronous callback.
    ObservableGauge,
}

impl InstrumentKind {
    /// Select the [Temporality] preference based on [InstrumentKind]
    ///
    /// [exporter-docs]: https://github.com/open-telemetry/opentelemetry-specification/blob/a1c13d59bb7d0fb086df2b3e1eaec9df9efef6cc/specification/metrics/sdk_exporters/otlp.md#additional-configuration
    pub(crate) fn temporality_preference(&self, temporality: Temporality) -> Temporality {
        match temporality {
            Temporality::Cumulative => Temporality::Cumulative,
            Temporality::Delta => match self {
                Self::Counter
                | Self::Histogram
                | Self::ObservableCounter
                | Self::Gauge
                | Self::ObservableGauge => Temporality::Delta,
                Self::UpDownCounter | InstrumentKind::ObservableUpDownCounter => {
                    Temporality::Cumulative
                }
            },
            Temporality::LowMemory => match self {
                Self::Counter | InstrumentKind::Histogram => Temporality::Delta,
                Self::ObservableCounter
                | Self::Gauge
                | Self::ObservableGauge
                | Self::UpDownCounter
                | Self::ObservableUpDownCounter => Temporality::Cumulative,
            },
        }
    }
}

/// Describes properties an instrument is created with, also used for filtering
/// in [View](crate::metrics::View)s.
///
/// # Example
///
/// Instruments can be used as criteria for views.
///
/// ```
/// use opentelemetry_sdk::metrics::{new_view, Aggregation, Instrument, StreamBuilder};
///
/// let criteria = Instrument::new().name("counter_*");
/// let mask = Stream::builder()
///     .with_aggregation(Aggregation::Sum)
///     .build()
///     .unwrap();
///
/// let view = new_view(criteria, mask);
/// # drop(view);
/// ```
#[derive(Clone, Default, Debug, PartialEq)]
#[non_exhaustive]
pub struct Instrument {
    /// The human-readable identifier of the instrument.
    pub name: Cow<'static, str>,
    /// describes the purpose of the instrument.
    pub description: Cow<'static, str>,
    /// The functional group of the instrument.
    pub kind: Option<InstrumentKind>,
    /// Unit is the unit of measurement recorded by the instrument.
    pub unit: Cow<'static, str>,
    /// The instrumentation that created the instrument.
    pub scope: InstrumentationScope,
}

#[cfg(feature = "spec_unstable_metrics_views")]
impl Instrument {
    /// Create a new instrument with default values
    pub fn new() -> Self {
        Instrument::default()
    }

    /// Set the instrument name.
    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the instrument description.
    pub fn description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the instrument unit.
    pub fn unit(mut self, unit: impl Into<Cow<'static, str>>) -> Self {
        self.unit = unit.into();
        self
    }

    /// Set the instrument scope.
    pub fn scope(mut self, scope: InstrumentationScope) -> Self {
        self.scope = scope;
        self
    }

    /// empty returns if all fields of i are their default-value.
    pub(crate) fn is_empty(&self) -> bool {
        self.name.is_empty()
            && self.description.is_empty()
            && self.kind.is_none()
            && self.unit.is_empty()
            && self.scope == InstrumentationScope::default()
    }

    pub(crate) fn matches(&self, other: &Instrument) -> bool {
        self.matches_name(other)
            && self.matches_description(other)
            && self.matches_kind(other)
            && self.matches_unit(other)
            && self.matches_scope(other)
    }

    pub(crate) fn matches_name(&self, other: &Instrument) -> bool {
        self.name.is_empty() || self.name.as_ref() == other.name.as_ref()
    }

    pub(crate) fn matches_description(&self, other: &Instrument) -> bool {
        self.description.is_empty() || self.description.as_ref() == other.description.as_ref()
    }

    pub(crate) fn matches_kind(&self, other: &Instrument) -> bool {
        self.kind.is_none() || self.kind == other.kind
    }

    pub(crate) fn matches_unit(&self, other: &Instrument) -> bool {
        self.unit.is_empty() || self.unit.as_ref() == other.unit.as_ref()
    }

    pub(crate) fn matches_scope(&self, other: &Instrument) -> bool {
        (self.scope.name().is_empty() || self.scope.name() == other.scope.name())
            && (self.scope.version().is_none()
                || self.scope.version().as_ref() == other.scope.version().as_ref())
            && (self.scope.schema_url().is_none()
                || self.scope.schema_url().as_ref() == other.scope.schema_url().as_ref())
    }
}

/// A builder for creating Stream objects.
///
/// # Example
///
/// ```
/// use opentelemetry_sdk::metrics::{Aggregation, StreamBuilder};
/// use opentelemetry::Key;
///
/// let stream = StreamBuilder::new()
///     .with_name("my_stream")
///     .with_aggregation(Aggregation::Sum)
///     .with_cardinality_limit(100)
///     .build()
///     .unwrap();
/// ```
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct StreamBuilder {
    name: Option<Cow<'static, str>>,
    description: Option<Cow<'static, str>>,
    unit: Option<Cow<'static, str>>,
    aggregation: Option<Aggregation>,
    allowed_attribute_keys: Option<Arc<HashSet<Key>>>,
    cardinality_limit: Option<usize>,
}

impl StreamBuilder {
    /// Create a new stream builder with default values.
    pub(crate) fn new() -> Self {
        StreamBuilder::default()
    }

    /// Set the stream name. If this is not set, name provide while creating the instrument will be used.
    pub fn with_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the stream description. If this is not set, description provided while creating the instrument will be used.
    pub fn with_description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the stream unit. If this is not set, unit provided while creating the instrument will be used.
    pub fn with_unit(mut self, unit: impl Into<Cow<'static, str>>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    #[cfg(feature = "spec_unstable_metrics_views")]
    /// Set the stream aggregation. This is used to customize the aggregation.
    /// If not set, the default aggregation based on the instrument kind will be used.
    pub fn with_aggregation(mut self, aggregation: Aggregation) -> Self {
        self.aggregation = Some(aggregation);
        self
    }

    #[cfg(feature = "spec_unstable_metrics_views")]
    /// Set the stream allowed attribute keys.
    ///
    /// Any attribute recorded for the stream with a key not in this set will be
    /// dropped. If the set is empty, all attributes will be dropped, if `None` all
    /// attributes will be kept.
    pub fn with_allowed_attribute_keys(
        mut self,
        attribute_keys: impl IntoIterator<Item = Key>,
    ) -> Self {
        self.allowed_attribute_keys = Some(Arc::new(attribute_keys.into_iter().collect()));
        self
    }

    /// Set the stream cardinality limit. If this is not set, the default limit of 2000 will be used.
    pub fn with_cardinality_limit(mut self, limit: usize) -> Self {
        self.cardinality_limit = Some(limit);
        self
    }

    /// Build a new Stream instance using the configuration in this builder.
    ///
    /// # Returns
    ///
    /// A Result containing the new Stream instance or an error if the build failed.
    pub fn build(self) -> Result<Stream, Box<dyn Error>> {
        // TODO: Add same validation as already done while
        // creating instruments. It is better to move validation logic
        // to a common helper and call it from both places.
        // The current implementations does a basic validation
        // only to close the overall API design.

        // if name is provided, it must not be empty
        if let Some(name) = &self.name {
            if name.is_empty() {
                return Err("Stream name must not be empty".into());
            }
        }

        // if cardinality limit is provided, it must be greater than 0
        if let Some(limit) = self.cardinality_limit {
            if limit == 0 {
                return Err("Cardinality limit must be greater than 0".into());
            }
        }

        // If the aggregation is set to Histogram, validate the bucket boundaries.
        if let Some(aggregation) = &self.aggregation {
            if let Aggregation::ExplicitBucketHistogram { boundaries, .. } = aggregation {
                validate_bucket_boundaries(boundaries)?;
            }
        }

        Ok(Stream {
            name: self.name,
            description: self.description,
            unit: self.unit,
            aggregation: self.aggregation,
            allowed_attribute_keys: self.allowed_attribute_keys,
            cardinality_limit: self.cardinality_limit,
        })
    }
}

fn validate_bucket_boundaries(boundaries: &[f64]) -> Result<(), String> {
    // Validate boundaries do not contain f64::NAN, f64::INFINITY, or f64::NEG_INFINITY
    for boundary in boundaries {
        if boundary.is_nan() || boundary.is_infinite() {
            return Err(
                "Bucket boundaries must not contain NaN, Infinity, or -Infinity".to_string(),
            );
        }
    }

    // validate that buckets are sorted and non-duplicate
    for i in 1..boundaries.len() {
        if boundaries[i] <= boundaries[i - 1] {
            return Err("Bucket boundaries must be sorted and non-duplicate".to_string());
        }
    }

    Ok(())
}

/// Describes the stream of data an instrument produces.
///
/// # Example
///
/// Streams can be used as masks in views.
///
/// ```
/// use opentelemetry_sdk::metrics::{new_view, Aggregation, Instrument, StreamBuilder};
///
/// let criteria = Instrument::new().name("counter_*");
/// let mask = StreamBuilder::new()
///     .with_aggregation(Aggregation::Sum)
///     .build()
///     .unwrap();
///
/// let view = new_view(criteria, mask);
/// # drop(view);
/// ```
#[derive(Default, Debug)]
#[non_exhaustive]
#[allow(unreachable_pub)]
pub struct Stream {
    /// The human-readable identifier of the stream.
    pub(crate) name: Option<Cow<'static, str>>,
    /// Describes the purpose of the data.
    pub(crate) description: Option<Cow<'static, str>>,
    /// the unit of measurement recorded.
    pub(crate) unit: Option<Cow<'static, str>>,
    /// Aggregation the stream uses for an instrument.
    pub(crate) aggregation: Option<Aggregation>,
    /// An allow-list of attribute keys that will be preserved for the stream.
    ///
    /// Any attribute recorded for the stream with a key not in this set will be
    /// dropped. If the set is empty, all attributes will be dropped, if `None` all
    /// attributes will be kept.
    pub(crate) allowed_attribute_keys: Option<Arc<HashSet<Key>>>,

    /// Cardinality limit for the stream.
    pub(crate) cardinality_limit: Option<usize>,
}

impl Stream {
    /// Create a new stream builder with default values.
    pub fn builder() -> StreamBuilder {
        StreamBuilder::new()
    }
}

/// The identifying properties of an instrument.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct InstrumentId {
    /// The human-readable identifier of the instrument.
    pub(crate) name: Cow<'static, str>,
    /// Describes the purpose of the data.
    pub(crate) description: Cow<'static, str>,
    /// Defines the functional group of the instrument.
    pub(crate) kind: InstrumentKind,
    /// The unit of measurement recorded.
    pub(crate) unit: Cow<'static, str>,
    /// Number is the underlying data type of the instrument.
    pub(crate) number: Cow<'static, str>,
}

impl InstrumentId {
    /// Instrument names are considered case-insensitive ASCII.
    ///
    /// Standardize the instrument name to always be lowercase so it can be compared
    /// via hash.
    ///
    /// See [naming syntax] for full requirements.
    ///
    /// [naming syntax]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.21.0/specification/metrics/api.md#instrument-name-syntax
    pub(crate) fn normalize(&mut self) {
        if self.name.chars().any(|c| c.is_ascii_uppercase()) {
            self.name = self.name.to_ascii_lowercase().into();
        }
    }
}

pub(crate) struct ResolvedMeasures<T> {
    pub(crate) measures: Vec<Arc<dyn Measure<T>>>,
}

impl<T: Copy + 'static> SyncInstrument<T> for ResolvedMeasures<T> {
    fn measure(&self, val: T, attrs: &[KeyValue]) {
        for measure in &self.measures {
            measure.call(val, attrs)
        }
    }
}

#[derive(Clone)]
pub(crate) struct Observable<T> {
    measures: Vec<Arc<dyn Measure<T>>>,
}

impl<T> Observable<T> {
    pub(crate) fn new(measures: Vec<Arc<dyn Measure<T>>>) -> Self {
        Self { measures }
    }
}

impl<T: Copy + Send + Sync + 'static> AsyncInstrument<T> for Observable<T> {
    fn observe(&self, measurement: T, attrs: &[KeyValue]) {
        for measure in &self.measures {
            measure.call(measurement, attrs)
        }
    }
}
