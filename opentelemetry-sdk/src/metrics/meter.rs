use core::fmt;
use std::{borrow::Cow, sync::Arc};

use opentelemetry::{
    global,
    metrics::{
        noop::{NoopAsyncInstrument, NoopSyncInstrument},
        AsyncInstrumentBuilder, Counter, Gauge, Histogram, HistogramBuilder, InstrumentBuilder,
        InstrumentProvider, MetricsError, ObservableCounter, ObservableGauge,
        ObservableUpDownCounter, Result, UpDownCounter,
    },
};

use crate::instrumentation::Scope;
use crate::metrics::{
    instrument::{Instrument, InstrumentKind, Observable, ResolvedMeasures},
    internal::{self, Number},
    pipeline::{Pipelines, Resolver},
};

// maximum length of instrument name
const INSTRUMENT_NAME_MAX_LENGTH: usize = 255;
// maximum length of instrument unit name
const INSTRUMENT_UNIT_NAME_MAX_LENGTH: usize = 63;
const INSTRUMENT_NAME_ALLOWED_NON_ALPHANUMERIC_CHARS: [char; 4] = ['_', '.', '-', '/'];

// instrument validation error strings
const INSTRUMENT_NAME_EMPTY: &str = "instrument name must be non-empty";
const INSTRUMENT_NAME_LENGTH: &str = "instrument name must be less than 256 characters";
const INSTRUMENT_NAME_INVALID_CHAR: &str =
    "characters in instrument name must be ASCII and belong to the alphanumeric characters, '_', '.', '-' and '/'";
const INSTRUMENT_NAME_FIRST_ALPHABETIC: &str =
    "instrument name must start with an alphabetic character";
const INSTRUMENT_UNIT_LENGTH: &str = "instrument unit must be less than 64 characters";
const INSTRUMENT_UNIT_INVALID_CHAR: &str = "characters in instrument unit must be ASCII";

/// Handles the creation and coordination of all metric instruments.
///
/// A meter represents a single instrumentation scope; all metric telemetry
/// produced by an instrumentation scope will use metric instruments from a
/// single meter.
///
/// See the [Meter API] docs for usage.
///
/// [Meter API]: opentelemetry::metrics::Meter
pub(crate) struct SdkMeter {
    scope: Scope,
    pipes: Arc<Pipelines>,
    u64_resolver: Resolver<u64>,
    i64_resolver: Resolver<i64>,
    f64_resolver: Resolver<f64>,
}

impl SdkMeter {
    pub(crate) fn new(scope: Scope, pipes: Arc<Pipelines>) -> Self {
        let view_cache = Default::default();

        SdkMeter {
            scope,
            pipes: Arc::clone(&pipes),
            u64_resolver: Resolver::new(Arc::clone(&pipes), Arc::clone(&view_cache)),
            i64_resolver: Resolver::new(Arc::clone(&pipes), Arc::clone(&view_cache)),
            f64_resolver: Resolver::new(pipes, view_cache),
        }
    }

    fn create_counter<T>(
        &self,
        builder: InstrumentBuilder<'_, Counter<T>>,
        resolver: &InstrumentResolver<'_, T>,
    ) -> Result<Counter<T>>
    where
        T: Number,
    {
        let validation_result = validate_instrument_config(builder.name.as_ref(), &builder.unit);
        if let Err(err) = validation_result {
            global::handle_error(err);
            return Ok(Counter::new(Arc::new(NoopSyncInstrument::new())));
        }

        match resolver
            .lookup(
                InstrumentKind::Counter,
                builder.name,
                builder.description,
                builder.unit,
                None,
            )
            .map(|i| Counter::new(Arc::new(i)))
        {
            Ok(counter) => Ok(counter),
            Err(err) => {
                global::handle_error(err);
                Ok(Counter::new(Arc::new(NoopSyncInstrument::new())))
            }
        }
    }

    fn create_observable_counter<T>(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableCounter<T>, T>,
        resolver: &InstrumentResolver<'_, T>,
    ) -> Result<ObservableCounter<T>>
    where
        T: Number,
    {
        let validation_result = validate_instrument_config(builder.name.as_ref(), &builder.unit);
        if let Err(err) = validation_result {
            global::handle_error(err);
            return Ok(ObservableCounter::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let ms = resolver.measures(
            InstrumentKind::ObservableCounter,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )?;

        if ms.is_empty() {
            return Ok(ObservableCounter::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(ms));

        for callback in builder.callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableCounter::new(observable))
    }

    fn create_observable_updown_counter<T>(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<T>, T>,
        resolver: &InstrumentResolver<'_, T>,
    ) -> Result<ObservableUpDownCounter<T>>
    where
        T: Number,
    {
        let validation_result = validate_instrument_config(builder.name.as_ref(), &builder.unit);
        if let Err(err) = validation_result {
            global::handle_error(err);
            return Ok(ObservableUpDownCounter::new(Arc::new(
                NoopAsyncInstrument::new(),
            )));
        }

        let ms = resolver.measures(
            InstrumentKind::ObservableUpDownCounter,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )?;

        if ms.is_empty() {
            return Ok(ObservableUpDownCounter::new(Arc::new(
                NoopAsyncInstrument::new(),
            )));
        }

        let observable = Arc::new(Observable::new(ms));

        for callback in builder.callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableUpDownCounter::new(observable))
    }

    fn create_observable_gauge<T>(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<T>, T>,
        resolver: &InstrumentResolver<'_, T>,
    ) -> Result<ObservableGauge<T>>
    where
        T: Number,
    {
        let validation_result = validate_instrument_config(builder.name.as_ref(), &builder.unit);
        if let Err(err) = validation_result {
            global::handle_error(err);
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let ms = resolver.measures(
            InstrumentKind::ObservableGauge,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )?;

        if ms.is_empty() {
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(ms));

        for callback in builder.callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableGauge::new(observable))
    }

    fn create_updown_counter<T>(
        &self,
        builder: InstrumentBuilder<'_, UpDownCounter<T>>,
        resolver: &InstrumentResolver<'_, T>,
    ) -> Result<UpDownCounter<T>>
    where
        T: Number,
    {
        let validation_result = validate_instrument_config(builder.name.as_ref(), &builder.unit);
        if let Err(err) = validation_result {
            global::handle_error(err);
            return Ok(UpDownCounter::new(Arc::new(NoopSyncInstrument::new())));
        }

        match resolver
            .lookup(
                InstrumentKind::UpDownCounter,
                builder.name,
                builder.description,
                builder.unit,
                None,
            )
            .map(|i| UpDownCounter::new(Arc::new(i)))
        {
            Ok(updown_counter) => Ok(updown_counter),
            Err(err) => {
                global::handle_error(err);
                Ok(UpDownCounter::new(Arc::new(NoopSyncInstrument::new())))
            }
        }
    }

    fn create_gauge<T>(
        &self,
        builder: InstrumentBuilder<'_, Gauge<T>>,
        resolver: &InstrumentResolver<'_, T>,
    ) -> Result<Gauge<T>>
    where
        T: Number,
    {
        let validation_result = validate_instrument_config(builder.name.as_ref(), &builder.unit);
        if let Err(err) = validation_result {
            global::handle_error(err);
            return Ok(Gauge::new(Arc::new(NoopSyncInstrument::new())));
        }

        match resolver
            .lookup(
                InstrumentKind::Gauge,
                builder.name,
                builder.description,
                builder.unit,
                None,
            )
            .map(|i| Gauge::new(Arc::new(i)))
        {
            Ok(gauge) => Ok(gauge),
            Err(err) => {
                global::handle_error(err);
                Ok(Gauge::new(Arc::new(NoopSyncInstrument::new())))
            }
        }
    }

    fn create_histogram<T>(
        &self,
        builder: HistogramBuilder<'_, T>,
        resolver: &InstrumentResolver<'_, T>,
    ) -> Result<Histogram<T>>
    where
        T: Number,
    {
        let validation_result = validate_instrument_config(builder.name.as_ref(), &builder.unit);
        if let Err(err) = validation_result {
            global::handle_error(err);
            return Ok(Histogram::new(Arc::new(NoopSyncInstrument::new())));
        }

        match resolver
            .lookup(
                InstrumentKind::Histogram,
                builder.name,
                builder.description,
                builder.unit,
                builder.boundaries,
            )
            .map(|i| Histogram::new(Arc::new(i)))
        {
            Ok(histogram) => Ok(histogram),
            Err(err) => {
                global::handle_error(err);
                Ok(Histogram::new(Arc::new(NoopSyncInstrument::new())))
            }
        }
    }
}

#[doc(hidden)]
impl InstrumentProvider for SdkMeter {
    fn u64_counter(&self, builder: InstrumentBuilder<'_, Counter<u64>>) -> Result<Counter<u64>> {
        let resolver = InstrumentResolver::new(self, &self.u64_resolver);
        self.create_counter(builder, &resolver)
    }

    fn f64_counter(&self, builder: InstrumentBuilder<'_, Counter<f64>>) -> Result<Counter<f64>> {
        let resolver = InstrumentResolver::new(self, &self.f64_resolver);
        self.create_counter(builder, &resolver)
    }

    fn u64_observable_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64>,
    ) -> Result<ObservableCounter<u64>> {
        let resolver = InstrumentResolver::new(self, &self.u64_resolver);
        self.create_observable_counter(builder, &resolver)
    }

    fn f64_observable_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64>,
    ) -> Result<ObservableCounter<f64>> {
        let resolver = InstrumentResolver::new(self, &self.f64_resolver);
        self.create_observable_counter(builder, &resolver)
    }

    fn i64_up_down_counter(
        &self,
        builder: InstrumentBuilder<'_, UpDownCounter<i64>>,
    ) -> Result<UpDownCounter<i64>> {
        let resolver = InstrumentResolver::new(self, &self.i64_resolver);
        self.create_updown_counter(builder, &resolver)
    }

    fn f64_up_down_counter(
        &self,
        builder: InstrumentBuilder<'_, UpDownCounter<f64>>,
    ) -> Result<UpDownCounter<f64>> {
        let resolver = InstrumentResolver::new(self, &self.f64_resolver);
        self.create_updown_counter(builder, &resolver)
    }

    fn i64_observable_up_down_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64>,
    ) -> Result<ObservableUpDownCounter<i64>> {
        let resolver = InstrumentResolver::new(self, &self.i64_resolver);
        self.create_observable_updown_counter(builder, &resolver)
    }

    fn f64_observable_up_down_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64>,
    ) -> Result<ObservableUpDownCounter<f64>> {
        let resolver = InstrumentResolver::new(self, &self.f64_resolver);
        self.create_observable_updown_counter(builder, &resolver)
    }

    fn u64_gauge(&self, builder: InstrumentBuilder<'_, Gauge<u64>>) -> Result<Gauge<u64>> {
        let resolver = InstrumentResolver::new(self, &self.u64_resolver);
        self.create_gauge(builder, &resolver)
    }

    fn f64_gauge(&self, builder: InstrumentBuilder<'_, Gauge<f64>>) -> Result<Gauge<f64>> {
        let resolver = InstrumentResolver::new(self, &self.f64_resolver);
        self.create_gauge(builder, &resolver)
    }

    fn i64_gauge(&self, builder: InstrumentBuilder<'_, Gauge<i64>>) -> Result<Gauge<i64>> {
        let resolver = InstrumentResolver::new(self, &self.i64_resolver);
        self.create_gauge(builder, &resolver)
    }

    fn u64_observable_gauge(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64>,
    ) -> Result<ObservableGauge<u64>> {
        let resolver = InstrumentResolver::new(self, &self.u64_resolver);
        self.create_observable_gauge(builder, &resolver)
    }

    fn i64_observable_gauge(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64>,
    ) -> Result<ObservableGauge<i64>> {
        let resolver = InstrumentResolver::new(self, &self.i64_resolver);
        self.create_observable_gauge(builder, &resolver)
    }

    fn f64_observable_gauge(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64>,
    ) -> Result<ObservableGauge<f64>> {
        let resolver = InstrumentResolver::new(self, &self.f64_resolver);
        self.create_observable_gauge(builder, &resolver)
    }

    fn f64_histogram(&self, builder: HistogramBuilder<'_, f64>) -> Result<Histogram<f64>> {
        let resolver = InstrumentResolver::new(self, &self.f64_resolver);
        self.create_histogram(builder, &resolver)
    }

    fn u64_histogram(&self, builder: HistogramBuilder<'_, u64>) -> Result<Histogram<u64>> {
        let resolver = InstrumentResolver::new(self, &self.u64_resolver);
        self.create_histogram(builder, &resolver)
    }
}

fn validate_instrument_config(name: &str, unit: &Option<Cow<'static, str>>) -> Result<()> {
    validate_instrument_name(name).and_then(|_| validate_instrument_unit(unit))
}

fn validate_instrument_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_EMPTY,
        ));
    }
    if name.len() > INSTRUMENT_NAME_MAX_LENGTH {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_LENGTH,
        ));
    }
    if name.starts_with(|c: char| !c.is_ascii_alphabetic()) {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_FIRST_ALPHABETIC,
        ));
    }
    if name.contains(|c: char| {
        !c.is_ascii_alphanumeric() && !INSTRUMENT_NAME_ALLOWED_NON_ALPHANUMERIC_CHARS.contains(&c)
    }) {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_INVALID_CHAR,
        ));
    }
    Ok(())
}

fn validate_instrument_unit(unit: &Option<Cow<'static, str>>) -> Result<()> {
    if let Some(unit) = unit {
        if unit.len() > INSTRUMENT_UNIT_NAME_MAX_LENGTH {
            return Err(MetricsError::InvalidInstrumentConfiguration(
                INSTRUMENT_UNIT_LENGTH,
            ));
        }
        if unit.contains(|c: char| !c.is_ascii()) {
            return Err(MetricsError::InvalidInstrumentConfiguration(
                INSTRUMENT_UNIT_INVALID_CHAR,
            ));
        }
    }
    Ok(())
}

impl fmt::Debug for SdkMeter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Meter").field("scope", &self.scope).finish()
    }
}

/// Provides all OpenTelemetry instruments.
struct InstrumentResolver<'a, T> {
    meter: &'a SdkMeter,
    resolve: &'a Resolver<T>,
}

impl<'a, T> InstrumentResolver<'a, T>
where
    T: Number,
{
    fn new(meter: &'a SdkMeter, resolve: &'a Resolver<T>) -> Self {
        InstrumentResolver { meter, resolve }
    }

    /// lookup returns the resolved measures.
    fn lookup(
        &self,
        kind: InstrumentKind,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Cow<'static, str>>,
        boundaries: Option<Vec<f64>>,
    ) -> Result<ResolvedMeasures<T>> {
        let aggregators = self.measures(kind, name, description, unit, boundaries)?;
        Ok(ResolvedMeasures {
            measures: aggregators,
        })
    }

    fn measures(
        &self,
        kind: InstrumentKind,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Cow<'static, str>>,
        boundaries: Option<Vec<f64>>,
    ) -> Result<Vec<Arc<dyn internal::Measure<T>>>> {
        let inst = Instrument {
            name,
            description: description.unwrap_or_default(),
            unit: unit.unwrap_or_default(),
            kind: Some(kind),
            scope: self.meter.scope.clone(),
        };

        self.resolve.measures(inst, boundaries)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use opentelemetry::metrics::MetricsError;

    use super::{
        validate_instrument_name, validate_instrument_unit, INSTRUMENT_NAME_FIRST_ALPHABETIC,
        INSTRUMENT_NAME_INVALID_CHAR, INSTRUMENT_NAME_LENGTH, INSTRUMENT_UNIT_INVALID_CHAR,
        INSTRUMENT_UNIT_LENGTH,
    };

    #[test]
    fn instrument_name_validation() {
        // (name, expected error)
        let instrument_name_test_cases = vec![
            ("validateName", ""),
            ("_startWithNoneAlphabet", INSTRUMENT_NAME_FIRST_ALPHABETIC),
            ("utf8char锈", INSTRUMENT_NAME_INVALID_CHAR),
            ("a".repeat(255).leak(), ""),
            ("a".repeat(256).leak(), INSTRUMENT_NAME_LENGTH),
            ("invalid name", INSTRUMENT_NAME_INVALID_CHAR),
            ("allow/slash", ""),
            ("allow_under_score", ""),
            ("allow.dots.ok", ""),
        ];
        for (name, expected_error) in instrument_name_test_cases {
            let assert = |result: Result<_, MetricsError>| {
                if expected_error.is_empty() {
                    assert!(result.is_ok());
                } else {
                    assert!(matches!(
                        result.unwrap_err(),
                        MetricsError::InvalidInstrumentConfiguration(msg) if msg == expected_error
                    ));
                }
            };

            assert(validate_instrument_name(name).map(|_| ()));
        }
    }

    #[test]
    fn instrument_unit_validation() {
        // (unit, expected error)
        let instrument_unit_test_cases = vec![
            (
                "0123456789012345678901234567890123456789012345678901234567890123",
                INSTRUMENT_UNIT_LENGTH,
            ),
            ("utf8char锈", INSTRUMENT_UNIT_INVALID_CHAR),
            ("kb", ""),
            ("Kb/sec", ""),
            ("%", ""),
            ("", ""),
        ];

        for (unit, expected_error) in instrument_unit_test_cases {
            let assert = |result: Result<_, MetricsError>| {
                if expected_error.is_empty() {
                    assert!(result.is_ok());
                } else {
                    assert!(matches!(
                        result.unwrap_err(),
                        MetricsError::InvalidInstrumentConfiguration(msg) if msg == expected_error
                    ));
                }
            };
            let unit: Option<Cow<'static, str>> = Some(unit.into());

            assert(validate_instrument_unit(&unit).map(|_| ()));
        }
    }
}
