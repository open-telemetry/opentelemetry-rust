use std::{any::Any, borrow::Cow, collections::HashSet, hash::Hash, sync::Arc};

use opentelemetry::{
    metrics::{AsyncInstrument, SyncCounter, SyncGauge, SyncHistogram, SyncUpDownCounter},
    Key, KeyValue,
};

use crate::{
    instrumentation::Scope,
    metrics::{aggregation::Aggregation, internal::Measure},
};

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

/// Describes properties an instrument is created with, also used for filtering
/// in [View](crate::metrics::View)s.
///
/// # Example
///
/// Instruments can be used as criteria for views.
///
/// ```
/// use opentelemetry_sdk::metrics::{new_view, Aggregation, Instrument, Stream};
///
/// let criteria = Instrument::new().name("counter_*");
/// let mask = Stream::new().aggregation(Aggregation::Sum);
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
    pub scope: Scope,
}

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
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// empty returns if all fields of i are their default-value.
    pub(crate) fn is_empty(&self) -> bool {
        self.name == ""
            && self.description == ""
            && self.kind.is_none()
            && self.unit == ""
            && self.scope == Scope::default()
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
        (self.scope.name.is_empty() || self.scope.name.as_ref() == other.scope.name.as_ref())
            && (self.scope.version.is_none()
                || self.scope.version.as_ref().map(AsRef::as_ref)
                    == other.scope.version.as_ref().map(AsRef::as_ref))
            && (self.scope.schema_url.is_none()
                || self.scope.schema_url.as_ref().map(AsRef::as_ref)
                    == other.scope.schema_url.as_ref().map(AsRef::as_ref))
    }
}

/// Describes the stream of data an instrument produces.
///
/// # Example
///
/// Streams can be used as masks in views.
///
/// ```
/// use opentelemetry_sdk::metrics::{new_view, Aggregation, Instrument, Stream};
///
/// let criteria = Instrument::new().name("counter_*");
/// let mask = Stream::new().aggregation(Aggregation::Sum);
///
/// let view = new_view(criteria, mask);
/// # drop(view);
/// ```
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct Stream {
    /// The human-readable identifier of the stream.
    pub name: Cow<'static, str>,
    /// Describes the purpose of the data.
    pub description: Cow<'static, str>,
    /// the unit of measurement recorded.
    pub unit: Cow<'static, str>,
    /// Aggregation the stream uses for an instrument.
    pub aggregation: Option<Aggregation>,
    /// An allow-list of attribute keys that will be preserved for the stream.
    ///
    /// Any attribute recorded for the stream with a key not in this set will be
    /// dropped. If the set is empty, all attributes will be dropped, if `None` all
    /// attributes will be kept.
    pub allowed_attribute_keys: Option<Arc<HashSet<Key>>>,
}

impl Stream {
    /// Create a new stream with empty values.
    pub fn new() -> Self {
        Stream::default()
    }

    /// Set the stream name.
    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the stream description.
    pub fn description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the stream unit.
    pub fn unit(mut self, unit: impl Into<Cow<'static, str>>) -> Self {
        self.unit = unit.into();
        self
    }

    /// Set the stream aggregation.
    pub fn aggregation(mut self, aggregation: Aggregation) -> Self {
        self.aggregation = Some(aggregation);
        self
    }

    /// Set the stream allowed attribute keys.
    ///
    /// Any attribute recorded for the stream with a key not in this set will be
    /// dropped. If this set is empty all attributes will be dropped.
    pub fn allowed_attribute_keys(mut self, attribute_keys: impl IntoIterator<Item = Key>) -> Self {
        self.allowed_attribute_keys = Some(Arc::new(attribute_keys.into_iter().collect()));

        self
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

impl<T: Copy + 'static> SyncCounter<T> for ResolvedMeasures<T> {
    fn add(&self, val: T, attrs: &[KeyValue]) {
        for measure in &self.measures {
            measure.call(val, attrs)
        }
    }
}

impl<T: Copy + 'static> SyncUpDownCounter<T> for ResolvedMeasures<T> {
    fn add(&self, val: T, attrs: &[KeyValue]) {
        for measure in &self.measures {
            measure.call(val, attrs)
        }
    }
}

impl<T: Copy + 'static> SyncGauge<T> for ResolvedMeasures<T> {
    fn record(&self, val: T, attrs: &[KeyValue]) {
        for measure in &self.measures {
            measure.call(val, attrs)
        }
    }
}

impl<T: Copy + 'static> SyncHistogram<T> for ResolvedMeasures<T> {
    fn record(&self, val: T, attrs: &[KeyValue]) {
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

    fn as_any(&self) -> Arc<dyn Any> {
        Arc::new(self.clone())
    }
}
