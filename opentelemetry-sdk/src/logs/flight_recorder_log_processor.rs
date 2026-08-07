use crate::error::{OTelSdkError, OTelSdkResult};
use crate::logs::{LogProcessor, SdkLogRecord};
use crate::Resource;
use opentelemetry::logs::Severity;
use opentelemetry::{otel_warn, InstrumentationScope};
use std::collections::VecDeque;
use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, TryLockError};
use std::time::{Duration, Instant};

const DEFAULT_MAX_RECORDS: usize = 2_048;
const DEFAULT_MAX_BUFFERED_SEVERITY: Severity = Severity::Info4;

type BufferedLog = (SdkLogRecord, InstrumentationScope);

/// A log processor that retains recent logs until explicitly triggered.
///
/// Records are held in an application-wide, per-processor bounded ring buffer.
/// When the buffer is full, the oldest record is overwritten. Calling
/// [`FlightRecorderTrigger::trigger`] drains the current snapshot into the
/// wrapped processor and then force-flushes it. Calling
/// [`LogProcessor::force_flush`] has the same effect.
///
/// By default, records in the TRACE, DEBUG, and INFO severity ranges are
/// buffered, while WARN, ERROR, and FATAL records bypass the buffer and are
/// immediately handed to the wrapped processor. Records without a severity are
/// buffered. The threshold can be changed with
/// [`FlightRecorderLogProcessorBuilder::with_max_buffered_severity`].
///
/// When wrapping a [`crate::logs::BatchLogProcessor`], configure its queue to
/// have at least `max_records` free slots when a trigger starts. Triggers are
/// serialized and each trigger flushes the wrapped processor before another
/// snapshot is replayed, so a queue of at least `max_records` is sufficient when
/// this flight recorder is its only producer.
///
/// Snapshot handoff is at-most-once and best-effort. [`LogProcessor::emit`]
/// cannot report whether a wrapped processor accepted a record, so records
/// rejected by an undersized or full delegate queue cannot be restored.
///
/// Untriggered records are discarded during shutdown.
pub struct FlightRecorderLogProcessor<P: LogProcessor> {
    shared: Arc<Shared<P>>,
}

impl<P: LogProcessor> Debug for FlightRecorderLogProcessor<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlightRecorderLogProcessor")
            .field("max_records", &self.shared.max_records)
            .field("max_buffered_severity", &self.shared.max_buffered_severity)
            .finish_non_exhaustive()
    }
}

impl<P: LogProcessor> FlightRecorderLogProcessor<P> {
    /// Creates a builder wrapping `delegate`.
    pub fn builder(delegate: P) -> FlightRecorderLogProcessorBuilder<P> {
        FlightRecorderLogProcessorBuilder {
            delegate,
            max_records: DEFAULT_MAX_RECORDS,
            max_buffered_severity: DEFAULT_MAX_BUFFERED_SEVERITY,
        }
    }
}

/// Configures a [`FlightRecorderLogProcessor`].
#[derive(Debug)]
pub struct FlightRecorderLogProcessorBuilder<P: LogProcessor> {
    delegate: P,
    max_records: usize,
    max_buffered_severity: Severity,
}

impl<P: LogProcessor + 'static> FlightRecorderLogProcessorBuilder<P> {
    /// Sets the maximum number of records retained by the flight recorder.
    ///
    /// # Panics
    ///
    /// [`build`](Self::build) panics if `max_records` is zero.
    pub fn with_max_records(mut self, max_records: usize) -> Self {
        self.max_records = max_records;
        self
    }

    /// Sets the highest severity retained by the flight recorder.
    ///
    /// Records with a higher severity bypass the ring buffer and are handed
    /// directly to the wrapped processor. Records without a severity are
    /// buffered.
    pub fn with_max_buffered_severity(mut self, severity: Severity) -> Self {
        self.max_buffered_severity = severity;
        self
    }

    /// Builds the processor and its cloneable trigger handle.
    ///
    /// # Panics
    ///
    /// Panics if `max_records` is zero.
    pub fn build(self) -> (FlightRecorderLogProcessor<P>, FlightRecorderTrigger) {
        assert!(
            self.max_records > 0,
            "flight recorder max_records must be greater than zero"
        );

        let shared = Arc::new(Shared {
            delegate: RwLock::new(self.delegate),
            state: Mutex::new(State {
                records: VecDeque::new(),
                is_shutdown: false,
            }),
            trigger_lock: Mutex::new(()),
            max_records: self.max_records,
            max_buffered_severity: self.max_buffered_severity,
        });
        let trigger = FlightRecorderTrigger {
            inner: shared.clone(),
        };

        (FlightRecorderLogProcessor { shared }, trigger)
    }
}

/// A cloneable handle that exports the flight recorder's current snapshot.
#[derive(Clone)]
pub struct FlightRecorderTrigger {
    inner: Arc<dyn TriggerFlightRecorder>,
}

impl FlightRecorderTrigger {
    /// Drains and exports the current flight-recorder snapshot.
    ///
    /// Concurrent triggers are serialized. Records emitted while a snapshot is
    /// being replayed are retained for the next trigger. A successful result
    /// means the wrapped processor's `force_flush` succeeded; it cannot confirm
    /// that every individual `emit` was accepted by the wrapped processor.
    pub fn trigger(&self) -> OTelSdkResult {
        self.inner.trigger()
    }
}

impl Debug for FlightRecorderTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlightRecorderTrigger")
            .finish_non_exhaustive()
    }
}

trait TriggerFlightRecorder: Send + Sync {
    fn trigger(&self) -> OTelSdkResult;
}

struct Shared<P: LogProcessor> {
    delegate: RwLock<P>,
    state: Mutex<State>,
    trigger_lock: Mutex<()>,
    max_records: usize,
    max_buffered_severity: Severity,
}

struct State {
    records: VecDeque<BufferedLog>,
    is_shutdown: bool,
}

impl<P: LogProcessor> Shared<P> {
    fn trigger_and_flush(&self) -> OTelSdkResult {
        let _trigger_guard = self
            .trigger_lock
            .lock()
            .map_err(|err| mutex_error("trigger", err))?;

        let delegate = self
            .delegate
            .write()
            .map_err(|err| lock_error("delegate", err))?;

        // Export records that bypassed the recorder before replaying the
        // contextual snapshot.
        delegate.force_flush()?;

        let snapshot = {
            let mut state = self
                .state
                .lock()
                .map_err(|err| mutex_error("buffer", err))?;
            if state.is_shutdown {
                return Err(OTelSdkError::AlreadyShutdown);
            }
            std::mem::take(&mut state.records)
        };

        if snapshot.is_empty() {
            return Ok(());
        }
        for (mut record, instrumentation) in snapshot {
            delegate.emit(&mut record, &instrumentation);
        }
        delegate.force_flush()
    }

    fn lock_trigger_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<MutexGuard<'_, ()>, OTelSdkError> {
        let start = Instant::now();
        loop {
            match self.trigger_lock.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(TryLockError::Poisoned(err)) => {
                    return Err(mutex_error("trigger", err));
                }
                Err(TryLockError::WouldBlock) => {
                    let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
                        return Err(OTelSdkError::Timeout(timeout));
                    };
                    if remaining.is_zero() {
                        return Err(OTelSdkError::Timeout(timeout));
                    }
                    std::thread::sleep(remaining.min(Duration::from_millis(1)));
                }
            }
        }
    }
}

impl<P: LogProcessor> TriggerFlightRecorder for Shared<P> {
    fn trigger(&self) -> OTelSdkResult {
        self.trigger_and_flush()
    }
}

impl<P: LogProcessor> LogProcessor for FlightRecorderLogProcessor<P> {
    fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        if record
            .severity_number()
            .is_some_and(|severity| severity > self.shared.max_buffered_severity)
        {
            let delegate = match self.shared.delegate.read() {
                Ok(delegate) => delegate,
                Err(err) => {
                    otel_warn!(
                        name: "FlightRecorderLogProcessor.DelegateLockFailed",
                        error = format!("{err}")
                    );
                    return;
                }
            };
            let state = match self.shared.state.lock() {
                Ok(state) => state,
                Err(err) => {
                    otel_warn!(
                        name: "FlightRecorderLogProcessor.BufferLockFailed",
                        error = format!("{err}")
                    );
                    return;
                }
            };
            if state.is_shutdown {
                otel_warn!(
                    name: "FlightRecorderLogProcessor.EmitAfterShutdown",
                    message = "FlightRecorderLogProcessor dropped a log emitted after shutdown."
                );
                return;
            }
            drop(state);
            delegate.emit(record, instrumentation);
            return;
        }

        let buffered_log = (record.clone(), instrumentation.clone());
        let mut state = match self.shared.state.lock() {
            Ok(state) => state,
            Err(err) => {
                otel_warn!(
                    name: "FlightRecorderLogProcessor.BufferLockFailed",
                    error = format!("{err}")
                );
                return;
            }
        };

        if state.is_shutdown {
            otel_warn!(
                name: "FlightRecorderLogProcessor.EmitAfterShutdown",
                message = "FlightRecorderLogProcessor dropped a log emitted after shutdown."
            );
            return;
        }

        if state.records.len() == self.shared.max_records {
            state.records.pop_front();
        }
        state.records.push_back(buffered_log);
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.shared.trigger_and_flush()
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        let start = Instant::now();
        let _trigger_guard = self.shared.lock_trigger_with_timeout(timeout)?;

        {
            let mut state = self
                .shared
                .state
                .lock()
                .map_err(|err| mutex_error("buffer", err))?;
            if state.is_shutdown {
                return Err(OTelSdkError::AlreadyShutdown);
            }
            state.is_shutdown = true;
            state.records.clear();
        }

        let remaining = timeout.saturating_sub(start.elapsed());
        self.shared
            .delegate
            .write()
            .map_err(|err| lock_error("delegate", err))?
            .shutdown_with_timeout(remaining)
    }

    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        match self.shared.delegate.read() {
            Ok(delegate) => delegate.event_enabled(level, target, name),
            Err(err) => {
                otel_warn!(
                    name: "FlightRecorderLogProcessor.DelegateLockFailed",
                    error = format!("{err}")
                );
                true
            }
        }
    }

    fn set_resource(&mut self, resource: &Resource) {
        match self.shared.delegate.write() {
            Ok(mut delegate) => delegate.set_resource(resource),
            Err(err) => {
                otel_warn!(
                    name: "FlightRecorderLogProcessor.DelegateLockFailed",
                    error = format!("{err}")
                );
            }
        }
    }
}

fn mutex_error<T>(name: &str, err: std::sync::PoisonError<T>) -> OTelSdkError {
    OTelSdkError::InternalFailure(format!(
        "FlightRecorderLogProcessor {name} mutex poisoned: {err}"
    ))
}

fn lock_error<T>(name: &str, err: std::sync::PoisonError<T>) -> OTelSdkError {
    OTelSdkError::InternalFailure(format!(
        "FlightRecorderLogProcessor {name} lock poisoned: {err}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::logs::{AnyValue, LogRecord};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Barrier;

    #[derive(Debug, Clone)]
    struct TestProcessor {
        records: Arc<Mutex<Vec<BufferedLog>>>,
        flushes: Arc<AtomicUsize>,
        shutdown: Arc<AtomicBool>,
        enabled: bool,
        resource: Arc<Mutex<Option<Resource>>>,
    }

    impl TestProcessor {
        fn new(enabled: bool) -> Self {
            Self {
                records: Arc::new(Mutex::new(Vec::new())),
                flushes: Arc::new(AtomicUsize::new(0)),
                shutdown: Arc::new(AtomicBool::new(false)),
                enabled,
                resource: Arc::new(Mutex::new(None)),
            }
        }
    }

    impl LogProcessor for TestProcessor {
        fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
            self.records
                .lock()
                .unwrap()
                .push((record.clone(), instrumentation.clone()));
        }

        fn force_flush(&self) -> OTelSdkResult {
            self.flushes.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
            self.shutdown.store(true, Ordering::Relaxed);
            Ok(())
        }

        fn event_enabled(&self, _level: Severity, _target: &str, _name: Option<&str>) -> bool {
            self.enabled
        }

        fn set_resource(&mut self, resource: &Resource) {
            *self.resource.lock().unwrap() = Some(resource.clone());
        }
    }

    #[derive(Debug, Clone)]
    struct BlockingProcessor {
        records: Arc<Mutex<Vec<BufferedLog>>>,
        emit_started: Arc<Barrier>,
        release_emit: Arc<Barrier>,
        block_next_emit: Arc<AtomicBool>,
    }

    impl LogProcessor for BlockingProcessor {
        fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
            if self.block_next_emit.swap(false, Ordering::Relaxed) {
                self.emit_started.wait();
                self.release_emit.wait();
            }
            self.records
                .lock()
                .unwrap()
                .push((record.clone(), instrumentation.clone()));
        }

        fn force_flush(&self) -> OTelSdkResult {
            Ok(())
        }

        fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
            Ok(())
        }
    }

    fn record(body: &'static str) -> SdkLogRecord {
        let mut record = SdkLogRecord::new();
        record.set_body(AnyValue::from(body));
        record
    }

    fn record_with_severity(body: &'static str, severity: Severity) -> SdkLogRecord {
        let mut record = record(body);
        record.set_severity_number(severity);
        record
    }

    fn bodies(processor: &TestProcessor) -> Vec<String> {
        processor
            .records
            .lock()
            .unwrap()
            .iter()
            .map(|(record, _)| match record.body() {
                Some(AnyValue::String(value)) => value.to_string(),
                body => panic!("unexpected body: {body:?}"),
            })
            .collect()
    }

    #[test]
    fn records_are_exported_only_when_triggered() {
        let delegate = TestProcessor::new(true);
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record("one"), &scope);
        processor.emit(&mut record("two"), &scope);
        assert!(bodies(&delegate).is_empty());

        trigger.trigger().unwrap();
        assert_eq!(bodies(&delegate), ["one", "two"]);
        assert_eq!(delegate.flushes.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn oldest_records_are_overwritten() {
        let delegate = TestProcessor::new(true);
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_records(2)
            .build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record("one"), &scope);
        processor.emit(&mut record("two"), &scope);
        processor.emit(&mut record("three"), &scope);
        trigger.trigger().unwrap();

        assert_eq!(bodies(&delegate), ["two", "three"]);
    }

    #[test]
    fn trigger_starts_a_new_snapshot() {
        let delegate = TestProcessor::new(true);
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record("one"), &scope);
        trigger.trigger().unwrap();
        processor.emit(&mut record("two"), &scope);
        trigger.trigger().unwrap();

        assert_eq!(bodies(&delegate), ["one", "two"]);
        assert_eq!(delegate.flushes.load(Ordering::Relaxed), 4);
    }

    #[test]
    fn records_emitted_during_replay_remain_for_the_next_trigger() {
        let delegate = BlockingProcessor {
            records: Arc::new(Mutex::new(Vec::new())),
            emit_started: Arc::new(Barrier::new(2)),
            release_emit: Arc::new(Barrier::new(2)),
            block_next_emit: Arc::new(AtomicBool::new(true)),
        };
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record("one"), &scope);
        let first_trigger = trigger.clone();
        let trigger_thread = std::thread::spawn(move || first_trigger.trigger());
        delegate.emit_started.wait();

        processor.emit(&mut record("two"), &scope);
        delegate.release_emit.wait();
        trigger_thread.join().unwrap().unwrap();

        assert_eq!(
            delegate.records.lock().unwrap()[0].0.body(),
            Some(&AnyValue::from("one"))
        );
        trigger.trigger().unwrap();
        let exported = delegate.records.lock().unwrap();
        assert_eq!(exported.len(), 2);
        assert_eq!(exported[1].0.body(), Some(&AnyValue::from("two")));
    }

    #[test]
    fn concurrent_triggers_do_not_duplicate_records() {
        let delegate = TestProcessor::new(true);
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();
        processor.emit(&mut record("one"), &scope);

        let first = trigger.clone();
        let second = trigger.clone();
        let first_thread = std::thread::spawn(move || first.trigger());
        let second_thread = std::thread::spawn(move || second.trigger());
        first_thread.join().unwrap().unwrap();
        second_thread.join().unwrap().unwrap();

        assert_eq!(bodies(&delegate), ["one"]);
        assert_eq!(delegate.flushes.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn force_flush_exports_the_snapshot() {
        let delegate = TestProcessor::new(true);
        let (processor, _trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record("one"), &scope);
        processor.force_flush().unwrap();

        assert_eq!(bodies(&delegate), ["one"]);
        assert_eq!(delegate.flushes.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn higher_severity_records_bypass_the_buffer() {
        let delegate = TestProcessor::new(true);
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record_with_severity("info", Severity::Info), &scope);
        processor.emit(&mut record_with_severity("warn", Severity::Warn), &scope);

        assert_eq!(bodies(&delegate), ["warn"]);
        trigger.trigger().unwrap();
        assert_eq!(bodies(&delegate), ["warn", "info"]);
    }

    #[test]
    fn higher_severity_records_are_dropped_after_shutdown() {
        let delegate = TestProcessor::new(true);
        let (processor, _trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();

        processor
            .shutdown_with_timeout(Duration::from_secs(1))
            .unwrap();
        processor.emit(&mut record_with_severity("warn", Severity::Warn), &scope);

        assert!(bodies(&delegate).is_empty());
    }

    #[test]
    fn buffered_severity_threshold_is_configurable() {
        let delegate = TestProcessor::new(true);
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_buffered_severity(Severity::Warn4)
            .build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record_with_severity("warn", Severity::Warn), &scope);
        processor.emit(&mut record_with_severity("error", Severity::Error), &scope);

        assert_eq!(bodies(&delegate), ["error"]);
        trigger.trigger().unwrap();
        assert_eq!(bodies(&delegate), ["error", "warn"]);
    }

    #[test]
    fn shutdown_discards_untriggered_records() {
        let delegate = TestProcessor::new(true);
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();

        processor.emit(&mut record("discarded"), &scope);
        processor
            .shutdown_with_timeout(Duration::from_secs(1))
            .unwrap();

        assert!(bodies(&delegate).is_empty());
        assert!(delegate.shutdown.load(Ordering::Relaxed));
        assert!(matches!(
            trigger.trigger(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
    }

    #[test]
    fn shutdown_timeout_includes_waiting_for_an_active_trigger() {
        let delegate = BlockingProcessor {
            records: Arc::new(Mutex::new(Vec::new())),
            emit_started: Arc::new(Barrier::new(2)),
            release_emit: Arc::new(Barrier::new(2)),
            block_next_emit: Arc::new(AtomicBool::new(true)),
        };
        let (processor, trigger) = FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = InstrumentationScope::builder("test").build();
        processor.emit(&mut record("one"), &scope);

        let trigger_thread = std::thread::spawn(move || trigger.trigger());
        delegate.emit_started.wait();
        assert!(matches!(
            processor.shutdown_with_timeout(Duration::from_millis(10)),
            Err(OTelSdkError::Timeout(_))
        ));

        delegate.release_emit.wait();
        trigger_thread.join().unwrap().unwrap();
        processor
            .shutdown_with_timeout(Duration::from_secs(1))
            .unwrap();
    }

    #[test]
    fn delegates_resource_and_event_enabled() {
        let delegate = TestProcessor::new(false);
        let (mut processor, _trigger) =
            FlightRecorderLogProcessor::builder(delegate.clone()).build();
        let resource = Resource::builder_empty().with_service_name("test").build();

        processor.set_resource(&resource);

        assert_eq!(*delegate.resource.lock().unwrap(), Some(resource));
        assert!(!processor.event_enabled(Severity::Info, "target", None));
    }

    #[test]
    #[should_panic(expected = "flight recorder max_records must be greater than zero")]
    fn zero_capacity_panics() {
        let delegate = TestProcessor::new(true);
        let _ = FlightRecorderLogProcessor::builder(delegate)
            .with_max_records(0)
            .build();
    }
}
