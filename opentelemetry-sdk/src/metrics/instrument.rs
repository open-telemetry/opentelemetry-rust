use std::{any::Any, borrow::Cow, fmt, hash::Hash, marker, sync::Arc};

use opentelemetry_api::{
    metrics::{
        AsyncInstrument, MetricsError, Result, SyncCounter, SyncHistogram, SyncUpDownCounter, Unit,
    },
    Context, KeyValue,
};

use crate::{
    instrumentation::Scope,
    metrics::data::Temporality,
    metrics::{aggregation::Aggregation, attributes::AttributeSet, internal::Aggregator},
};

pub(crate) const EMPTY_AGG_MSG: &str = "no aggregators for observable instrument";

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
    pub unit: Unit,
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
    pub fn unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
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
            && self.unit.as_str() == ""
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
        self.unit.as_str() == "" || self.unit == other.unit
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
#[derive(Default)]
#[non_exhaustive]
pub struct Stream {
    /// The human-readable identifier of the stream.
    pub name: Cow<'static, str>,
    /// Describes the purpose of the data.
    pub description: Cow<'static, str>,
    /// the unit of measurement recorded.
    pub unit: Unit,
    /// Aggregation the stream uses for an instrument.
    pub aggregation: Option<Aggregation>,
    /// applied to all attributes recorded for an instrument.
    pub attribute_filter: Option<Filter>,
}

type Filter = Arc<dyn Fn(&KeyValue) -> bool + Send + Sync>;

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
    pub fn unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }

    /// Set the stream aggregation.
    pub fn aggregation(mut self, aggregation: Aggregation) -> Self {
        self.aggregation = Some(aggregation);
        self
    }

    /// Set the stream attribute filter.
    pub fn attribute_filter(
        mut self,
        filter: impl Fn(&KeyValue) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.attribute_filter = Some(Arc::new(filter));
        self
    }
}

impl fmt::Debug for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stream")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("unit", &self.unit)
            .field("aggregation", &self.aggregation)
            .field("attribute_filter", &self.attribute_filter.is_some())
            .finish()
    }
}

/// the identifying properties of a stream.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct StreamId {
    /// The human-readable identifier of the stream.
    pub(crate) name: Cow<'static, str>,
    /// Describes the purpose of the data.
    pub(crate) description: Cow<'static, str>,
    /// the unit of measurement recorded.
    pub(crate) unit: Unit,
    /// The stream uses for an instrument.
    pub(crate) aggregation: String,
    /// Monotonic is the monotonicity of an instruments data type. This field is
    /// not used for all data types, so a zero value needs to be understood in the
    /// context of Aggregation.
    pub(crate) monotonic: bool,
    /// Temporality is the temporality of a stream's data type. This field is
    /// not used by some data types.
    pub(crate) temporality: Option<Temporality>,
    /// Number is the number type of the stream.
    pub(crate) number: Cow<'static, str>,
}

pub(crate) struct InstrumentImpl<T> {
    pub(crate) aggregators: Vec<Arc<dyn Aggregator<T>>>,
}

impl<T: Copy> SyncCounter<T> for InstrumentImpl<T> {
    fn add(&self, _cx: &Context, val: T, attrs: &[KeyValue]) {
        for agg in &self.aggregators {
            agg.aggregate(val, AttributeSet::from(attrs))
        }
    }
}

impl<T: Copy> SyncUpDownCounter<T> for InstrumentImpl<T> {
    fn add(&self, _cx: &Context, val: T, attrs: &[KeyValue]) {
        for agg in &self.aggregators {
            agg.aggregate(val, AttributeSet::from(attrs))
        }
    }
}

impl<T: Copy> SyncHistogram<T> for InstrumentImpl<T> {
    fn record(&self, _cx: &Context, val: T, attrs: &[KeyValue]) {
        for agg in &self.aggregators {
            agg.aggregate(val, AttributeSet::from(attrs))
        }
    }
}

/// A comparable unique identifier of an observable.
#[derive(Clone, Debug)]
pub(crate) struct ObservableId<T> {
    pub(crate) inner: IdInner,
    _marker: marker::PhantomData<T>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct IdInner {
    /// The human-readable identifier of the instrument.
    pub(crate) name: Cow<'static, str>,
    /// describes the purpose of the instrument.
    pub(crate) description: Cow<'static, str>,
    /// The functional group of the instrument.
    kind: InstrumentKind,
    /// Unit is the unit of measurement recorded by the instrument.
    pub(crate) unit: Unit,
    /// The instrumentation that created the instrument.
    scope: Scope,
}

impl<T> Hash for ObservableId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<T> PartialEq for ObservableId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for ObservableId<T> {}

#[derive(Clone)]
pub(crate) struct Observable<T> {
    pub(crate) id: ObservableId<T>,
    aggregators: Vec<Arc<dyn Aggregator<T>>>,
}

impl<T> Observable<T> {
    pub(crate) fn new(
        scope: Scope,
        kind: InstrumentKind,
        name: Cow<'static, str>,
        description: Cow<'static, str>,
        unit: Unit,
        aggregators: Vec<Arc<dyn Aggregator<T>>>,
    ) -> Self {
        Self {
            id: ObservableId {
                inner: IdInner {
                    name,
                    description,
                    kind,
                    unit,
                    scope,
                },
                _marker: marker::PhantomData,
            },
            aggregators,
        }
    }

    /// Returns `Err` if the observable should not be registered, and `Ok` if it
    /// should.
    ///
    /// An error is returned if this observable is effectively a no-op because it does not have
    /// any aggregators. Also, an error is returned if scope defines a Meter other
    /// than the observable it was created by.
    pub(crate) fn registerable(&self, scope: &Scope) -> Result<()> {
        if self.aggregators.is_empty() {
            return Err(MetricsError::Other(EMPTY_AGG_MSG.into()));
        }
        if &self.id.inner.scope != scope {
            return Err(MetricsError::Other(format!(
                "invalid registration: observable {} from Meter {:?}, registered with Meter {}",
                self.id.inner.name, self.id.inner.scope, scope.name,
            )));
        }

        Ok(())
    }
}

impl<T: Copy + Send + Sync + 'static> AsyncInstrument<T> for Observable<T> {
    fn observe(&self, measurement: T, attrs: &[KeyValue]) {
        for agg in &self.aggregators {
            agg.aggregate(measurement, AttributeSet::from(attrs))
        }
    }

    fn as_any(&self) -> Arc<dyn Any> {
        Arc::new(self.clone())
    }
}
