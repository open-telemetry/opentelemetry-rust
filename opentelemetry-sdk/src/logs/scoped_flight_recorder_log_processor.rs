use crate::error::{OTelSdkError, OTelSdkResult};
use crate::logs::flight_recorder::estimated_log_size;
use crate::logs::{LogProcessor, SdkLogRecord};
use crate::Resource;
use opentelemetry::logs::Severity;
use opentelemetry::{otel_warn, Context, InstrumentationScope};
use std::collections::{HashMap, VecDeque};
use std::fmt::{self, Debug, Formatter};
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, TryLockError, Weak};
use std::time::{Duration, Instant};

const DEFAULT_MAX_RECORDS_PER_SCOPE: usize = 256;
const DEFAULT_MAX_ACTIVE_SCOPES: usize = 128;
const DEFAULT_MAX_BUFFER_SIZE_BYTES_PER_SCOPE: usize = 256 * 1024;
const DEFAULT_MAX_TOTAL_BUFFER_SIZE_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_RECORD_SIZE_BYTES: usize = 64 * 1024;
const DEFAULT_MAX_BUFFERED_SEVERITY: Severity = Severity::Info4;

type BufferedLog = (SdkLogRecord, InstrumentationScope, usize);

static NEXT_PROCESSOR_ID: AtomicUsize = AtomicUsize::new(1);

/// A log processor that retains low-severity logs in operation-scoped buffers.
///
/// A scope is created with [`ScopedFlightRecorder::try_start`], then propagated
/// with [`ScopedFlightRecorderScope::with_context`]. TRACE, DEBUG, and INFO
/// records emitted inside that context are buffered by default. WARN and higher
/// records, logs outside an active scope, and logs emitted after a scope is
/// triggered follow the wrapped processor's normal path.
///
/// Dropping a scope without triggering it discards its buffered records.
pub struct ScopedFlightRecorderLogProcessor<P: LogProcessor> {
    shared: Arc<ScopedShared<P>>,
}

impl<P: LogProcessor> Debug for ScopedFlightRecorderLogProcessor<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopedFlightRecorderLogProcessor")
            .field("max_records_per_scope", &self.shared.max_records_per_scope)
            .field("max_active_scopes", &self.shared.max_active_scopes)
            .field(
                "max_buffer_size_bytes_per_scope",
                &self.shared.max_buffer_size_bytes_per_scope,
            )
            .field(
                "max_total_buffer_size_bytes",
                &self.shared.memory_budget.limit,
            )
            .field("max_record_size_bytes", &self.shared.max_record_size_bytes)
            .field("max_buffered_severity", &self.shared.max_buffered_severity)
            .finish_non_exhaustive()
    }
}

impl<P: LogProcessor> ScopedFlightRecorderLogProcessor<P> {
    /// Creates a builder wrapping `delegate`.
    pub fn builder(delegate: P) -> ScopedFlightRecorderLogProcessorBuilder<P> {
        ScopedFlightRecorderLogProcessorBuilder {
            delegate,
            max_records_per_scope: DEFAULT_MAX_RECORDS_PER_SCOPE,
            max_active_scopes: DEFAULT_MAX_ACTIVE_SCOPES,
            max_buffer_size_bytes_per_scope: DEFAULT_MAX_BUFFER_SIZE_BYTES_PER_SCOPE,
            max_total_buffer_size_bytes: DEFAULT_MAX_TOTAL_BUFFER_SIZE_BYTES,
            max_record_size_bytes: DEFAULT_MAX_RECORD_SIZE_BYTES,
            max_buffered_severity: DEFAULT_MAX_BUFFERED_SEVERITY,
        }
    }
}

/// Configures a [`ScopedFlightRecorderLogProcessor`].
#[derive(Debug)]
pub struct ScopedFlightRecorderLogProcessorBuilder<P: LogProcessor> {
    delegate: P,
    max_records_per_scope: usize,
    max_active_scopes: usize,
    max_buffer_size_bytes_per_scope: usize,
    max_total_buffer_size_bytes: usize,
    max_record_size_bytes: usize,
    max_buffered_severity: Severity,
}

impl<P: LogProcessor + 'static> ScopedFlightRecorderLogProcessorBuilder<P> {
    /// Sets the maximum number of records retained by each active scope.
    pub fn with_max_records_per_scope(mut self, max_records: usize) -> Self {
        self.max_records_per_scope = max_records;
        self
    }

    /// Sets the maximum number of simultaneously active recording scopes.
    pub fn with_max_active_scopes(mut self, max_active_scopes: usize) -> Self {
        self.max_active_scopes = max_active_scopes;
        self
    }

    /// Sets the maximum estimated memory retained by each active scope.
    pub fn with_max_buffer_size_bytes_per_scope(
        mut self,
        max_buffer_size_bytes_per_scope: usize,
    ) -> Self {
        self.max_buffer_size_bytes_per_scope = max_buffer_size_bytes_per_scope;
        self
    }

    /// Sets the aggregate estimated memory retained across all active scopes.
    pub fn with_max_total_buffer_size_bytes(mut self, max_total_buffer_size_bytes: usize) -> Self {
        self.max_total_buffer_size_bytes = max_total_buffer_size_bytes;
        self
    }

    /// Sets the maximum estimated size of an individual buffered record.
    ///
    /// Larger records bypass the recorder and follow the wrapped processor's
    /// normal path.
    pub fn with_max_record_size_bytes(mut self, max_record_size_bytes: usize) -> Self {
        self.max_record_size_bytes = max_record_size_bytes;
        self
    }

    /// Sets the highest severity retained in operation-scoped buffers.
    pub fn with_max_buffered_severity(mut self, severity: Severity) -> Self {
        self.max_buffered_severity = severity;
        self
    }

    /// Builds the processor and the handle used to create recording scopes.
    ///
    /// # Panics
    ///
    /// Panics if either configured capacity is zero.
    pub fn build(self) -> (ScopedFlightRecorderLogProcessor<P>, ScopedFlightRecorder) {
        assert!(
            self.max_records_per_scope > 0,
            "scoped flight recorder max_records_per_scope must be greater than zero"
        );
        assert!(
            self.max_active_scopes > 0,
            "scoped flight recorder max_active_scopes must be greater than zero"
        );
        assert!(
            self.max_buffer_size_bytes_per_scope > 0,
            "scoped flight recorder max_buffer_size_bytes_per_scope must be greater than zero"
        );
        assert!(
            self.max_total_buffer_size_bytes > 0,
            "scoped flight recorder max_total_buffer_size_bytes must be greater than zero"
        );
        assert!(
            self.max_record_size_bytes > 0,
            "scoped flight recorder max_record_size_bytes must be greater than zero"
        );

        let shared = Arc::new(ScopedShared {
            delegate: RwLock::new(self.delegate),
            trigger_lock: Mutex::new(()),
            scopes: Mutex::new(HashMap::new()),
            next_scope_id: AtomicUsize::new(1),
            is_shutdown: AtomicBool::new(false),
            processor_id: NEXT_PROCESSOR_ID.fetch_add(1, Ordering::Relaxed),
            max_records_per_scope: self.max_records_per_scope,
            max_active_scopes: self.max_active_scopes,
            max_buffer_size_bytes_per_scope: self.max_buffer_size_bytes_per_scope,
            max_record_size_bytes: self.max_record_size_bytes,
            memory_budget: Arc::new(MemoryBudget::new(self.max_total_buffer_size_bytes)),
            max_buffered_severity: self.max_buffered_severity,
        });
        let recorder = ScopedFlightRecorder {
            inner: shared.clone(),
        };

        (ScopedFlightRecorderLogProcessor { shared }, recorder)
    }
}

/// Creates operation-scoped flight-recorder buffers.
#[derive(Clone)]
pub struct ScopedFlightRecorder {
    inner: Arc<dyn ScopedFlightRecorderCore>,
}

impl ScopedFlightRecorder {
    /// Starts a recording scope.
    ///
    /// Returns `None` when the configured active-scope limit has been reached or
    /// the processor has shut down. Logs without a scope use the normal path.
    pub fn try_start(&self) -> Option<ScopedFlightRecorderScope> {
        self.inner
            .try_start()
            .map(|buffer| ScopedFlightRecorderScope {
                inner: self.inner.clone(),
                buffer,
            })
    }
}

impl Debug for ScopedFlightRecorder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopedFlightRecorder")
            .finish_non_exhaustive()
    }
}

/// An operation-scoped flight recorder.
#[derive(Clone)]
pub struct ScopedFlightRecorderScope {
    inner: Arc<dyn ScopedFlightRecorderCore>,
    buffer: Arc<ScopeBuffer>,
}

impl ScopedFlightRecorderScope {
    /// Runs `future` with this recording scope attached to the current
    /// OpenTelemetry context whenever the future is polled.
    ///
    /// Independently spawned tasks do not automatically inherit this context.
    /// Wrap those task futures with `with_context` as well when their logs
    /// should belong to the same scope.
    pub async fn with_context<F: Future>(&self, future: F) -> F::Output {
        let context = Context::current_with_value(ScopedBufferContext {
            buffer: self.buffer.clone(),
        });
        let mut future = pin!(future);
        poll_fn(|task_context| {
            let _guard = context.clone().attach();
            future.as_mut().poll(task_context)
        })
        .await
    }

    /// Exports this scope's buffered snapshot and switches it to passthrough.
    pub fn trigger(&self) -> OTelSdkResult {
        self.inner.trigger(&self.buffer)
    }

    /// Discards this scope's buffered records.
    pub fn discard(&self) {
        self.buffer.discard();
    }
}

impl Debug for ScopedFlightRecorderScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopedFlightRecorderScope")
            .field("id", &self.buffer.id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
struct ScopedBufferContext {
    buffer: Arc<ScopeBuffer>,
}

trait ScopedFlightRecorderCore: Send + Sync {
    fn try_start(&self) -> Option<Arc<ScopeBuffer>>;
    fn trigger(&self, buffer: &Arc<ScopeBuffer>) -> OTelSdkResult;
}

struct ScopeBuffer {
    id: usize,
    processor_id: usize,
    max_records: usize,
    max_buffer_size_bytes: usize,
    memory_budget: Arc<MemoryBudget>,
    accounted_bytes: AtomicUsize,
    state: Mutex<ScopeState>,
}

struct ScopeState {
    status: ScopeStatus,
    records: VecDeque<BufferedLog>,
    buffer_size_bytes: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeStatus {
    Recording,
    Passthrough,
    Discarded,
}

impl ScopeBuffer {
    fn try_buffer(
        &self,
        record: &SdkLogRecord,
        instrumentation: &InstrumentationScope,
        estimated_size: usize,
    ) -> Result<bool, OTelSdkError> {
        let mut state = self
            .state
            .lock()
            .map_err(|err| mutex_error("scope buffer", err))?;
        if state.status != ScopeStatus::Recording {
            return Ok(false);
        }
        let mut remaining_records = state.records.len();
        let mut remaining_bytes = state.buffer_size_bytes;
        let mut eviction_count = 0;
        let mut eviction_bytes = 0;
        for (_, _, size) in &state.records {
            if remaining_records < self.max_records
                && estimated_size <= self.max_buffer_size_bytes - remaining_bytes
            {
                break;
            }
            remaining_records -= 1;
            remaining_bytes -= size;
            eviction_count += 1;
            eviction_bytes += size;
        }

        let additional_bytes = estimated_size.saturating_sub(eviction_bytes);
        if !self.memory_budget.try_reserve(additional_bytes) {
            return Ok(false);
        }

        for _ in 0..eviction_count {
            state.records.pop_front();
        }
        if eviction_bytes > estimated_size {
            self.memory_budget.release(eviction_bytes - estimated_size);
        }
        state.buffer_size_bytes = remaining_bytes + estimated_size;
        self.accounted_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |accounted| {
                Some(accounted - eviction_bytes + estimated_size)
            })
            .expect("scope byte accounting must not underflow");
        state
            .records
            .push_back((record.clone(), instrumentation.clone(), estimated_size));
        Ok(true)
    }

    fn take_and_passthrough(&self) -> Result<BufferedSnapshot, OTelSdkError> {
        let mut state = self
            .state
            .lock()
            .map_err(|err| mutex_error("scope buffer", err))?;
        if state.status != ScopeStatus::Recording {
            return Ok(BufferedSnapshot::empty(self.memory_budget.clone()));
        }
        state.status = ScopeStatus::Passthrough;
        state.buffer_size_bytes = 0;
        self.accounted_bytes.store(0, Ordering::Release);
        Ok(BufferedSnapshot {
            records: std::mem::take(&mut state.records),
            memory_budget: self.memory_budget.clone(),
        })
    }

    fn discard(&self) {
        match self.state.lock() {
            Ok(mut state) => {
                state.status = ScopeStatus::Discarded;
                state.records.clear();
                state.buffer_size_bytes = 0;
                self.release_all_accounted_bytes();
            }
            Err(err) => {
                otel_warn!(
                    name: "ScopedFlightRecorderLogProcessor.BufferLockFailed",
                    error = format!("{err}")
                );
            }
        }
    }

    fn release_all_accounted_bytes(&self) {
        let bytes = self.accounted_bytes.swap(0, Ordering::AcqRel);
        self.memory_budget.release(bytes);
    }
}

struct BufferedSnapshot {
    records: VecDeque<BufferedLog>,
    memory_budget: Arc<MemoryBudget>,
}

impl BufferedSnapshot {
    fn empty(memory_budget: Arc<MemoryBudget>) -> Self {
        Self {
            records: VecDeque::new(),
            memory_budget,
        }
    }

    fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    fn pop_front(&mut self) -> Option<BufferedLog> {
        self.records.pop_front()
    }

    fn release(&self, bytes: usize) {
        self.memory_budget.release(bytes);
    }
}

impl Drop for BufferedSnapshot {
    fn drop(&mut self) {
        let bytes = self.records.iter().map(|(_, _, size)| size).sum();
        self.memory_budget.release(bytes);
    }
}

impl Drop for ScopeBuffer {
    fn drop(&mut self) {
        self.release_all_accounted_bytes();
    }
}

struct MemoryBudget {
    used: AtomicUsize,
    limit: usize,
}

impl MemoryBudget {
    fn new(limit: usize) -> Self {
        Self {
            used: AtomicUsize::new(0),
            limit,
        }
    }

    fn try_reserve(&self, bytes: usize) -> bool {
        if bytes == 0 {
            return true;
        }
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|total| *total <= self.limit)
            })
            .is_ok()
    }

    fn release(&self, bytes: usize) {
        if bytes > 0 {
            self.used.fetch_sub(bytes, Ordering::AcqRel);
        }
    }
}

struct ScopedShared<P: LogProcessor> {
    delegate: RwLock<P>,
    trigger_lock: Mutex<()>,
    scopes: Mutex<HashMap<usize, Weak<ScopeBuffer>>>,
    next_scope_id: AtomicUsize,
    is_shutdown: AtomicBool,
    processor_id: usize,
    max_records_per_scope: usize,
    max_active_scopes: usize,
    max_buffer_size_bytes_per_scope: usize,
    max_record_size_bytes: usize,
    memory_budget: Arc<MemoryBudget>,
    max_buffered_severity: Severity,
}

impl<P: LogProcessor> ScopedShared<P> {
    fn emit_to_delegate(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        let delegate = match self.delegate.read() {
            Ok(delegate) => delegate,
            Err(err) => {
                otel_warn!(
                    name: "ScopedFlightRecorderLogProcessor.DelegateLockFailed",
                    error = format!("{err}")
                );
                return;
            }
        };
        if self.is_shutdown.load(Ordering::Acquire) {
            otel_warn!(
                name: "ScopedFlightRecorderLogProcessor.EmitAfterShutdown",
                message = "ScopedFlightRecorderLogProcessor dropped a log emitted after shutdown."
            );
            return;
        }
        delegate.emit(record, instrumentation);
    }

    fn flush_snapshots(
        &self,
        buffers: impl IntoIterator<Item = Arc<ScopeBuffer>>,
    ) -> OTelSdkResult {
        let delegate = self
            .delegate
            .write()
            .map_err(|err| lock_error("delegate", err))?;
        delegate.force_flush()?;

        for buffer in buffers {
            let mut snapshot = buffer.take_and_passthrough()?;
            if snapshot.is_empty() {
                continue;
            }
            while let Some((mut record, instrumentation, estimated_size)) = snapshot.pop_front() {
                delegate.emit(&mut record, &instrumentation);
                snapshot.release(estimated_size);
            }
            // Bound each handoff to one scope so a delegate queue sized for
            // max_records_per_scope does not need capacity for every active
            // scope at once.
            delegate.force_flush()?;
        }

        Ok(())
    }

    fn active_buffers(&self) -> Result<Vec<Arc<ScopeBuffer>>, OTelSdkError> {
        let mut scopes = self
            .scopes
            .lock()
            .map_err(|err| mutex_error("scope registry", err))?;
        let mut buffers = Vec::with_capacity(scopes.len());
        scopes.retain(|_, buffer| {
            if let Some(buffer) = buffer.upgrade() {
                buffers.push(buffer);
                true
            } else {
                false
            }
        });
        Ok(buffers)
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

impl<P: LogProcessor + 'static> ScopedFlightRecorderCore for ScopedShared<P> {
    fn try_start(&self) -> Option<Arc<ScopeBuffer>> {
        if self.is_shutdown.load(Ordering::Acquire) {
            return None;
        }
        let mut scopes = self.scopes.lock().ok()?;
        scopes.retain(|_, buffer| buffer.strong_count() > 0);
        if self.is_shutdown.load(Ordering::Acquire) || scopes.len() >= self.max_active_scopes {
            return None;
        }

        let id = self.next_scope_id.fetch_add(1, Ordering::Relaxed);
        let buffer = Arc::new(ScopeBuffer {
            id,
            processor_id: self.processor_id,
            max_records: self.max_records_per_scope,
            max_buffer_size_bytes: self.max_buffer_size_bytes_per_scope,
            memory_budget: self.memory_budget.clone(),
            accounted_bytes: AtomicUsize::new(0),
            state: Mutex::new(ScopeState {
                status: ScopeStatus::Recording,
                records: VecDeque::new(),
                buffer_size_bytes: 0,
            }),
        });
        scopes.insert(id, Arc::downgrade(&buffer));
        Some(buffer)
    }

    fn trigger(&self, buffer: &Arc<ScopeBuffer>) -> OTelSdkResult {
        if self.is_shutdown.load(Ordering::Acquire) {
            return Err(OTelSdkError::AlreadyShutdown);
        }
        let _trigger_guard = self
            .trigger_lock
            .lock()
            .map_err(|err| mutex_error("trigger", err))?;
        if self.is_shutdown.load(Ordering::Acquire) {
            return Err(OTelSdkError::AlreadyShutdown);
        }
        self.flush_snapshots([buffer.clone()])
    }
}

impl<P: LogProcessor + 'static> LogProcessor for ScopedFlightRecorderLogProcessor<P> {
    fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        if self.shared.is_shutdown.load(Ordering::Acquire) {
            otel_warn!(
                name: "ScopedFlightRecorderLogProcessor.EmitAfterShutdown",
                message = "ScopedFlightRecorderLogProcessor dropped a log emitted after shutdown."
            );
            return;
        }

        let should_buffer = record.severity_number().map_or(true, |severity| {
            severity <= self.shared.max_buffered_severity
        });
        if should_buffer {
            let estimated_size = estimated_log_size(record, instrumentation);
            if estimated_size > self.shared.max_record_size_bytes
                || estimated_size > self.shared.max_buffer_size_bytes_per_scope
            {
                self.shared.emit_to_delegate(record, instrumentation);
                return;
            }
            let buffer = Context::map_current(|context| {
                context
                    .get::<ScopedBufferContext>()
                    .map(|context| context.buffer.clone())
            });
            if let Some(buffer) = buffer {
                if buffer.processor_id == self.shared.processor_id {
                    match buffer.try_buffer(record, instrumentation, estimated_size) {
                        Ok(true) => return,
                        Ok(false) => {}
                        Err(err) => {
                            otel_warn!(
                                name: "ScopedFlightRecorderLogProcessor.BufferLockFailed",
                                error = format!("{err}")
                            );
                            return;
                        }
                    }
                }
            }
        }

        self.shared.emit_to_delegate(record, instrumentation);
    }

    fn force_flush(&self) -> OTelSdkResult {
        let _trigger_guard = self
            .shared
            .trigger_lock
            .lock()
            .map_err(|err| mutex_error("trigger", err))?;
        if self.shared.is_shutdown.load(Ordering::Acquire) {
            return Err(OTelSdkError::AlreadyShutdown);
        }
        let buffers = self.shared.active_buffers()?;
        self.shared.flush_snapshots(buffers)
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        let start = Instant::now();
        let _trigger_guard = self.shared.lock_trigger_with_timeout(timeout)?;
        if self.shared.is_shutdown.swap(true, Ordering::AcqRel) {
            return Err(OTelSdkError::AlreadyShutdown);
        }
        for buffer in self.shared.active_buffers()? {
            buffer.discard();
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
                    name: "ScopedFlightRecorderLogProcessor.DelegateLockFailed",
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
                    name: "ScopedFlightRecorderLogProcessor.DelegateLockFailed",
                    error = format!("{err}")
                );
            }
        }
    }
}

fn mutex_error<T>(name: &str, err: std::sync::PoisonError<T>) -> OTelSdkError {
    OTelSdkError::InternalFailure(format!(
        "ScopedFlightRecorderLogProcessor {name} mutex poisoned: {err}"
    ))
}

fn lock_error<T>(name: &str, err: std::sync::PoisonError<T>) -> OTelSdkError {
    OTelSdkError::InternalFailure(format!(
        "ScopedFlightRecorderLogProcessor {name} lock poisoned: {err}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::logs::{AnyValue, LogRecord};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug, Clone)]
    struct TestProcessor {
        records: Arc<Mutex<Vec<BufferedLog>>>,
        flushes: Arc<AtomicUsize>,
    }

    #[derive(Debug, Clone)]
    struct BoundedQueueProcessor {
        pending: Arc<Mutex<Vec<BufferedLog>>>,
        exported: Arc<Mutex<Vec<BufferedLog>>>,
        dropped: Arc<AtomicUsize>,
        capacity: usize,
    }

    impl LogProcessor for BoundedQueueProcessor {
        fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
            let mut pending = self.pending.lock().unwrap();
            if pending.len() == self.capacity {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                return;
            }
            pending.push((record.clone(), instrumentation.clone(), 0));
        }

        fn force_flush(&self) -> OTelSdkResult {
            let mut pending = self.pending.lock().unwrap();
            self.exported.lock().unwrap().extend(pending.drain(..));
            Ok(())
        }

        fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
            Ok(())
        }
    }

    impl TestProcessor {
        fn new() -> Self {
            Self {
                records: Arc::new(Mutex::new(Vec::new())),
                flushes: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl LogProcessor for TestProcessor {
        fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
            self.records
                .lock()
                .unwrap()
                .push((record.clone(), instrumentation.clone(), 0));
        }

        fn force_flush(&self) -> OTelSdkResult {
            self.flushes.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
            Ok(())
        }
    }

    fn record(body: &'static str, severity: Severity) -> SdkLogRecord {
        let mut record = SdkLogRecord::new();
        record.set_body(AnyValue::from(body));
        record.set_severity_number(severity);
        record
    }

    fn bodies(processor: &TestProcessor) -> Vec<String> {
        processor
            .records
            .lock()
            .unwrap()
            .iter()
            .map(|(record, _, _)| match record.body() {
                Some(AnyValue::String(value)) => value.to_string(),
                body => panic!("unexpected body: {body:?}"),
            })
            .collect()
    }

    #[test]
    fn scopes_are_isolated() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let first = recorder.try_start().unwrap();
        let second = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(first.with_context(async {
            processor.emit(&mut record("first", Severity::Info), &instrumentation);
        }));
        futures_executor::block_on(second.with_context(async {
            processor.emit(&mut record("second", Severity::Info), &instrumentation);
        }));
        first.trigger().unwrap();

        assert_eq!(bodies(&delegate), ["first"]);
        second.discard();
    }

    #[test]
    fn logs_outside_a_scope_and_high_severity_logs_bypass() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        processor.emit(&mut record("outside", Severity::Info), &instrumentation);
        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("warn", Severity::Warn), &instrumentation);
            processor.emit(&mut record("info", Severity::Info), &instrumentation);
        }));

        assert_eq!(bodies(&delegate), ["outside", "warn"]);
        scope.trigger().unwrap();
        assert_eq!(bodies(&delegate), ["outside", "warn", "info"]);
    }

    #[test]
    fn triggered_scope_switches_to_passthrough() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("before", Severity::Info), &instrumentation);
        }));
        scope.trigger().unwrap();
        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("after", Severity::Info), &instrumentation);
        }));

        assert_eq!(bodies(&delegate), ["before", "after"]);
    }

    #[test]
    fn active_scope_limit_is_enforced_and_reclaimed() {
        let delegate = TestProcessor::new();
        let (_processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate)
            .with_max_active_scopes(1)
            .build();

        let first = recorder.try_start().unwrap();
        assert!(recorder.try_start().is_none());
        drop(first);
        assert!(recorder.try_start().is_some());
    }

    #[test]
    fn force_flush_triggers_all_active_scopes() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let first = recorder.try_start().unwrap();
        let second = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(first.with_context(async {
            processor.emit(&mut record("first", Severity::Info), &instrumentation);
        }));
        futures_executor::block_on(second.with_context(async {
            processor.emit(&mut record("second", Severity::Info), &instrumentation);
        }));
        processor.force_flush().unwrap();

        let mut exported = bodies(&delegate);
        exported.sort();
        assert_eq!(exported, ["first", "second"]);
    }

    #[test]
    fn force_flush_replays_each_scope_within_delegate_queue_capacity() {
        let delegate = BoundedQueueProcessor {
            pending: Arc::new(Mutex::new(Vec::new())),
            exported: Arc::new(Mutex::new(Vec::new())),
            dropped: Arc::new(AtomicUsize::new(0)),
            capacity: 2,
        };
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_records_per_scope(2)
            .build();
        let first = recorder.try_start().unwrap();
        let second = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(first.with_context(async {
            processor.emit(&mut record("first-1", Severity::Info), &instrumentation);
            processor.emit(&mut record("first-2", Severity::Info), &instrumentation);
        }));
        futures_executor::block_on(second.with_context(async {
            processor.emit(&mut record("second-1", Severity::Info), &instrumentation);
            processor.emit(&mut record("second-2", Severity::Info), &instrumentation);
        }));

        processor.force_flush().unwrap();

        assert_eq!(delegate.dropped.load(Ordering::Relaxed), 0);
        assert_eq!(delegate.exported.lock().unwrap().len(), 4);
    }

    #[test]
    fn scope_capacity_overwrites_oldest_records() {
        let delegate = TestProcessor::new();
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_records_per_scope(2)
            .build();
        let scope = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("one", Severity::Info), &instrumentation);
            processor.emit(&mut record("two", Severity::Info), &instrumentation);
            processor.emit(&mut record("three", Severity::Info), &instrumentation);
        }));
        scope.trigger().unwrap();

        assert_eq!(bodies(&delegate), ["two", "three"]);
    }

    #[test]
    fn scope_byte_capacity_overwrites_oldest_records() {
        let delegate = TestProcessor::new();
        let instrumentation = InstrumentationScope::builder("test").build();
        let estimated_size = estimated_log_size(&record("one", Severity::Info), &instrumentation);
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_buffer_size_bytes_per_scope(estimated_size * 2)
            .build();
        let scope = recorder.try_start().unwrap();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("one", Severity::Info), &instrumentation);
            processor.emit(&mut record("two", Severity::Info), &instrumentation);
            processor.emit(&mut record("six", Severity::Info), &instrumentation);
        }));
        scope.trigger().unwrap();

        assert_eq!(bodies(&delegate), ["two", "six"]);
    }

    #[test]
    fn aggregate_byte_capacity_is_shared_across_scopes() {
        let delegate = TestProcessor::new();
        let instrumentation = InstrumentationScope::builder("test").build();
        let estimated_size = estimated_log_size(&record("first", Severity::Info), &instrumentation);
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_buffer_size_bytes_per_scope(estimated_size * 2)
            .with_max_total_buffer_size_bytes(estimated_size)
            .build();
        let first = recorder.try_start().unwrap();
        let second = recorder.try_start().unwrap();

        futures_executor::block_on(first.with_context(async {
            processor.emit(&mut record("first", Severity::Info), &instrumentation);
        }));
        futures_executor::block_on(second.with_context(async {
            processor.emit(&mut record("other", Severity::Info), &instrumentation);
        }));
        assert_eq!(bodies(&delegate), ["other"]);

        first.discard();
        futures_executor::block_on(second.with_context(async {
            processor.emit(&mut record("third", Severity::Info), &instrumentation);
        }));
        second.trigger().unwrap();

        assert_eq!(bodies(&delegate), ["other", "third"]);
    }

    #[test]
    fn failed_aggregate_reservation_preserves_existing_snapshot() {
        const LARGE_BODY: &str =
            "a much larger record that requires more aggregate capacity than the existing record";

        let delegate = TestProcessor::new();
        let instrumentation = InstrumentationScope::builder("test").build();
        let small_size = estimated_log_size(&record("a", Severity::Info), &instrumentation);
        let large_size = estimated_log_size(&record(LARGE_BODY, Severity::Info), &instrumentation);
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_records_per_scope(1)
            .with_max_buffer_size_bytes_per_scope(large_size)
            .with_max_total_buffer_size_bytes(small_size * 2)
            .build();
        let first = recorder.try_start().unwrap();
        let second = recorder.try_start().unwrap();

        futures_executor::block_on(first.with_context(async {
            processor.emit(&mut record("a", Severity::Info), &instrumentation);
        }));
        futures_executor::block_on(second.with_context(async {
            processor.emit(&mut record("b", Severity::Info), &instrumentation);
        }));
        futures_executor::block_on(first.with_context(async {
            processor.emit(&mut record(LARGE_BODY, Severity::Info), &instrumentation);
        }));

        assert_eq!(bodies(&delegate), [LARGE_BODY]);
        first.trigger().unwrap();
        assert_eq!(bodies(&delegate), [LARGE_BODY, "a"]);
        second.discard();
    }

    #[test]
    fn aggregate_byte_capacity_is_released_by_scope_lifecycle() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let instrumentation = InstrumentationScope::builder("test").build();

        let discarded = recorder.try_start().unwrap();
        futures_executor::block_on(discarded.with_context(async {
            processor.emit(&mut record("discard", Severity::Info), &instrumentation);
        }));
        assert!(processor.shared.memory_budget.used.load(Ordering::Acquire) > 0);
        discarded.discard();
        assert_eq!(
            processor.shared.memory_budget.used.load(Ordering::Acquire),
            0
        );

        let triggered = recorder.try_start().unwrap();
        futures_executor::block_on(triggered.with_context(async {
            processor.emit(&mut record("trigger", Severity::Info), &instrumentation);
        }));
        triggered.trigger().unwrap();
        assert_eq!(
            processor.shared.memory_budget.used.load(Ordering::Acquire),
            0
        );

        let dropped = recorder.try_start().unwrap();
        futures_executor::block_on(dropped.with_context(async {
            processor.emit(&mut record("dropped", Severity::Info), &instrumentation);
        }));
        assert!(processor.shared.memory_budget.used.load(Ordering::Acquire) > 0);
        drop(dropped);
        assert_eq!(
            processor.shared.memory_budget.used.load(Ordering::Acquire),
            0
        );
    }

    #[test]
    fn oversized_scoped_records_bypass_the_buffer() {
        let delegate = TestProcessor::new();
        let instrumentation = InstrumentationScope::builder("test").build();
        let estimated_size =
            estimated_log_size(&record("oversized", Severity::Info), &instrumentation);
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_record_size_bytes(estimated_size - 1)
            .build();
        let scope = recorder.try_start().unwrap();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("oversized", Severity::Info), &instrumentation);
        }));

        assert_eq!(bodies(&delegate), ["oversized"]);
        scope.trigger().unwrap();
        assert_eq!(bodies(&delegate), ["oversized"]);
    }

    #[test]
    fn discarded_scope_does_not_export() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("discarded", Severity::Info), &instrumentation);
        }));
        scope.discard();
        scope.trigger().unwrap();

        assert!(bodies(&delegate).is_empty());
    }

    #[test]
    fn shutdown_discards_scopes_and_prevents_new_ones() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("discarded", Severity::Info), &instrumentation);
        }));
        processor
            .shutdown_with_timeout(Duration::from_secs(1))
            .unwrap();

        assert!(recorder.try_start().is_none());
        assert!(matches!(
            scope.trigger(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert!(bodies(&delegate).is_empty());
    }
}
