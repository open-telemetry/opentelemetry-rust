use core::fmt;
use std::{borrow::Cow, sync::Arc};

use opentelemetry::{
    global,
    metrics::{
        noop::NoopAsyncInstrument, AsyncInstrumentBuilder, Counter, Gauge, Histogram,
        HistogramBuilder, InstrumentBuilder, InstrumentProvider, MetricsError, ObservableCounter,
        ObservableGauge, ObservableUpDownCounter, Result, UpDownCounter,
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
    validation_policy: InstrumentValidationPolicy,
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
            validation_policy: InstrumentValidationPolicy::HandleGlobalAndIgnore,
        }
    }

    #[cfg(test)]
    fn with_validation_policy(self, validation_policy: InstrumentValidationPolicy) -> Self {
        Self {
            validation_policy,
            ..self
        }
    }
}

#[doc(hidden)]
impl InstrumentProvider for SdkMeter {
    fn u64_counter(&self, builder: InstrumentBuilder<'_, Counter<u64>>) -> Result<Counter<u64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.u64_resolver);
        p.lookup(
            InstrumentKind::Counter,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )
        .map(|i| Counter::new(Arc::new(i)))
    }

    fn f64_counter(&self, builder: InstrumentBuilder<'_, Counter<f64>>) -> Result<Counter<f64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.f64_resolver);
        p.lookup(
            InstrumentKind::Counter,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )
        .map(|i| Counter::new(Arc::new(i)))
    }

    fn u64_observable_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64>,
    ) -> Result<ObservableCounter<u64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.u64_resolver);
        let ms = p.measures(
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

    fn f64_observable_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64>,
    ) -> Result<ObservableCounter<f64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.f64_resolver);
        let ms = p.measures(
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

    fn i64_up_down_counter(
        &self,
        builder: InstrumentBuilder<'_, UpDownCounter<i64>>,
    ) -> Result<UpDownCounter<i64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.i64_resolver);
        p.lookup(
            InstrumentKind::UpDownCounter,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )
        .map(|i| UpDownCounter::new(Arc::new(i)))
    }

    fn f64_up_down_counter(
        &self,
        builder: InstrumentBuilder<'_, UpDownCounter<f64>>,
    ) -> Result<UpDownCounter<f64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.f64_resolver);
        p.lookup(
            InstrumentKind::UpDownCounter,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )
        .map(|i| UpDownCounter::new(Arc::new(i)))
    }

    fn i64_observable_up_down_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64>,
    ) -> Result<ObservableUpDownCounter<i64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.i64_resolver);
        let ms = p.measures(
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

    fn f64_observable_up_down_counter(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64>,
    ) -> Result<ObservableUpDownCounter<f64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.f64_resolver);
        let ms = p.measures(
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

    fn u64_gauge(&self, builder: InstrumentBuilder<'_, Gauge<u64>>) -> Result<Gauge<u64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.u64_resolver);
        p.lookup(
            InstrumentKind::Gauge,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )
        .map(|i| Gauge::new(Arc::new(i)))
    }

    fn f64_gauge(&self, builder: InstrumentBuilder<'_, Gauge<f64>>) -> Result<Gauge<f64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.f64_resolver);
        p.lookup(
            InstrumentKind::Gauge,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )
        .map(|i| Gauge::new(Arc::new(i)))
    }

    fn i64_gauge(&self, builder: InstrumentBuilder<'_, Gauge<i64>>) -> Result<Gauge<i64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.i64_resolver);
        p.lookup(
            InstrumentKind::Gauge,
            builder.name,
            builder.description,
            builder.unit,
            None,
        )
        .map(|i| Gauge::new(Arc::new(i)))
    }

    fn u64_observable_gauge(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64>,
    ) -> Result<ObservableGauge<u64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.u64_resolver);
        let ms = p.measures(
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

    fn i64_observable_gauge(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64>,
    ) -> Result<ObservableGauge<i64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.i64_resolver);
        let ms = p.measures(
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

    fn f64_observable_gauge(
        &self,
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64>,
    ) -> Result<ObservableGauge<f64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.f64_resolver);
        let ms = p.measures(
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

    fn f64_histogram(&self, builder: HistogramBuilder<'_, f64>) -> Result<Histogram<f64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.f64_resolver);
        p.lookup(
            InstrumentKind::Histogram,
            builder.name,
            builder.description,
            builder.unit,
            builder.boundaries,
        )
        .map(|i| Histogram::new(Arc::new(i)))
    }

    fn u64_histogram(&self, builder: HistogramBuilder<'_, u64>) -> Result<Histogram<u64>> {
        validate_instrument_config(builder.name.as_ref(), &builder.unit, self.validation_policy)?;
        let p = InstrumentResolver::new(self, &self.u64_resolver);
        p.lookup(
            InstrumentKind::Histogram,
            builder.name,
            builder.description,
            builder.unit,
            builder.boundaries,
        )
        .map(|i| Histogram::new(Arc::new(i)))
    }
}

/// Validation policy for instrument
#[derive(Clone, Copy)]
enum InstrumentValidationPolicy {
    HandleGlobalAndIgnore,
    /// Currently only for test
    #[cfg(test)]
    Strict,
}

fn validate_instrument_config(
    name: &str,
    unit: &Option<Cow<'static, str>>,
    policy: InstrumentValidationPolicy,
) -> Result<()> {
    match validate_instrument_name(name).and_then(|_| validate_instrument_unit(unit)) {
        Ok(_) => Ok(()),
        Err(err) => match policy {
            InstrumentValidationPolicy::HandleGlobalAndIgnore => {
                global::handle_error(err);
                Ok(())
            }
            #[cfg(test)]
            InstrumentValidationPolicy::Strict => Err(err),
        },
    }
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
    use std::{borrow::Cow, sync::Arc};

    use opentelemetry::{
        global,
        metrics::{InstrumentProvider, MeterProvider, MetricsError},
    };

    use super::{
        InstrumentValidationPolicy, SdkMeter, INSTRUMENT_NAME_FIRST_ALPHABETIC,
        INSTRUMENT_NAME_INVALID_CHAR, INSTRUMENT_NAME_LENGTH, INSTRUMENT_UNIT_INVALID_CHAR,
        INSTRUMENT_UNIT_LENGTH,
    };
    use crate::{
        metrics::{pipeline::Pipelines, SdkMeterProvider},
        Resource, Scope,
    };

    #[test]
    #[ignore = "See issue https://github.com/open-telemetry/opentelemetry-rust/issues/1699"]
    fn test_instrument_creation() {
        let provider = SdkMeterProvider::builder().build();
        let meter = provider.meter("test");
        assert!(meter.u64_counter("test").try_init().is_ok());
        let result = meter.u64_counter("test with invalid name").try_init();
        // this assert fails, as result is always ok variant.
        assert!(result.is_err());
    }

    #[test]
    fn test_instrument_config_validation() {
        // scope and pipelines are not related to test
        let meter = SdkMeter::new(
            Scope::default(),
            Arc::new(Pipelines::new(Resource::default(), Vec::new(), Vec::new())),
        )
        .with_validation_policy(InstrumentValidationPolicy::Strict);
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

            // Get handle to InstrumentBuilder for testing
            let global_meter = global::meter("test");
            let counter_builder_u64 = global_meter.u64_counter(name);
            let counter_builder_f64 = global_meter.f64_counter(name);

            // Get handle to AsyncInstrumentBuilder for testing
            let observable_counter_u64 = global_meter.u64_observable_counter(name);
            let observable_counter_f64 = global_meter.f64_observable_counter(name);

            assert(meter.u64_counter(counter_builder_u64).map(|_| ()));
            assert(meter.f64_counter(counter_builder_f64).map(|_| ()));
            assert(
                meter
                    .u64_observable_counter(observable_counter_u64)
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_counter(observable_counter_f64)
                    .map(|_| ()),
            );

            // Get handle to InstrumentBuilder for testing
            let up_down_counter_builder_i64 = global_meter.i64_up_down_counter(name);
            let up_down_counter_builder_f64 = global_meter.f64_up_down_counter(name);

            assert(
                meter
                    .i64_up_down_counter(up_down_counter_builder_i64)
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_up_down_counter(up_down_counter_builder_f64)
                    .map(|_| ()),
            );

            // Get handle to AsyncInstrumentBuilder for testing
            let observable_up_down_counter_i64 = global_meter.i64_observable_up_down_counter(name);
            let observable_up_down_counter_f64 = global_meter.f64_observable_up_down_counter(name);

            assert(
                meter
                    .i64_observable_up_down_counter(observable_up_down_counter_i64)
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_up_down_counter(observable_up_down_counter_f64)
                    .map(|_| ()),
            );

            // Get handle to InstrumentBuilder for testing
            let gauge_builder_u64 = global_meter.u64_gauge(name);
            let gauge_builder_f64 = global_meter.f64_gauge(name);
            let gauge_builder_i64 = global_meter.i64_gauge(name);

            assert(meter.u64_gauge(gauge_builder_u64).map(|_| ()));
            assert(meter.f64_gauge(gauge_builder_f64).map(|_| ()));
            assert(meter.i64_gauge(gauge_builder_i64).map(|_| ()));

            // Get handle to AsyncInstrumentBuilder for testing
            let observable_gauge_u64 = global_meter.u64_observable_gauge(name);
            let observable_gauge_i64 = global_meter.i64_observable_gauge(name);
            let observable_gauge_f64 = global_meter.f64_observable_gauge(name);

            assert(meter.u64_observable_gauge(observable_gauge_u64).map(|_| ()));
            assert(meter.i64_observable_gauge(observable_gauge_i64).map(|_| ()));
            assert(meter.f64_observable_gauge(observable_gauge_f64).map(|_| ()));

            // Get handle to HistogramBuilder for testing
            let histogram_builder_f64 = global_meter.f64_histogram(name);
            let histogram_builder_u64 = global_meter.u64_histogram(name);

            assert(meter.f64_histogram(histogram_builder_f64).map(|_| ()));
            assert(meter.u64_histogram(histogram_builder_u64).map(|_| ()));
        }

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

            // Get handle to InstrumentBuilder for testing
            let global_meter = global::meter("test");
            let counter_builder_u64 = global_meter
                .u64_counter("test")
                .with_unit(unit.clone().unwrap());
            let counter_builder_f64 = global_meter
                .f64_counter("test")
                .with_unit(unit.clone().unwrap());

            assert(meter.u64_counter(counter_builder_u64).map(|_| ()));
            assert(meter.f64_counter(counter_builder_f64).map(|_| ()));

            // Get handle to AsyncInstrumentBuilder for testing
            let observable_counter_u64 = global_meter
                .u64_observable_counter("test")
                .with_unit(unit.clone().unwrap());
            let observable_counter_f64 = global_meter
                .f64_observable_counter("test")
                .with_unit(unit.clone().unwrap());

            assert(
                meter
                    .u64_observable_counter(observable_counter_u64)
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_counter(observable_counter_f64)
                    .map(|_| ()),
            );

            // Get handle to InstrumentBuilder for testing
            let up_down_counter_builder_i64 = global_meter
                .i64_up_down_counter("test")
                .with_unit(unit.clone().unwrap());
            let up_down_counter_builder_f64 = global_meter
                .f64_up_down_counter("test")
                .with_unit(unit.clone().unwrap());

            assert(
                meter
                    .i64_up_down_counter(up_down_counter_builder_i64)
                    .map(|_| ()),
            );

            assert(
                meter
                    .f64_up_down_counter(up_down_counter_builder_f64)
                    .map(|_| ()),
            );

            // Get handle to AsyncInstrumentBuilder for testing
            let observable_up_down_counter_i64 = global_meter
                .i64_observable_up_down_counter("test")
                .with_unit(unit.clone().unwrap());
            let observable_up_down_counter_f64 = global_meter
                .f64_observable_up_down_counter("test")
                .with_unit(unit.clone().unwrap());

            assert(
                meter
                    .i64_observable_up_down_counter(observable_up_down_counter_i64)
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_up_down_counter(observable_up_down_counter_f64)
                    .map(|_| ()),
            );

            // Get handle to AsyncInstrumentBuilder for testing
            let observable_gauge_u64 = global_meter
                .u64_observable_gauge("test")
                .with_unit(unit.clone().unwrap());
            let observable_gauge_i64 = global_meter
                .i64_observable_gauge("test")
                .with_unit(unit.clone().unwrap());
            let observable_gauge_f64 = global_meter
                .f64_observable_gauge("test")
                .with_unit(unit.clone().unwrap());

            assert(meter.u64_observable_gauge(observable_gauge_u64).map(|_| ()));
            assert(meter.i64_observable_gauge(observable_gauge_i64).map(|_| ()));
            assert(meter.f64_observable_gauge(observable_gauge_f64).map(|_| ()));

            // Get handle to HistogramBuilder for testing
            let histogram_builder_f64 = global_meter
                .f64_histogram("test")
                .with_unit(unit.clone().unwrap());
            let histogram_builder_u64 = global_meter
                .u64_histogram("test")
                .with_unit(unit.clone().unwrap());

            assert(meter.f64_histogram(histogram_builder_f64).map(|_| ()));
            assert(meter.u64_histogram(histogram_builder_u64).map(|_| ()));
        }
    }
}
