use crate::error::{OTelSdkError, OTelSdkResult};
use crate::logs::flight_recorder::{
    estimated_log_size, lock_with_timeout, should_buffer, BufferedLog, FlightRecorderMetrics,
    LogBuffer, TimedLockError,
};
use crate::logs::{LogProcessor, SdkLogRecord};
use crate::Resource;
use opentelemetry::logs::Severity;
use opentelemetry::{otel_warn, Context, InstrumentationScope};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::{Duration, Instant};

const DEFAULT_MAX_RECORDS_PER_SCOPE: usize = 256;
const DEFAULT_MAX_ACTIVE_SCOPES: usize = 128;
const DEFAULT_MAX_BUFFER_SIZE_BYTES_PER_SCOPE: usize = 256 * 1024;
const DEFAULT_MAX_TOTAL_BUFFER_SIZE_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_RECORD_SIZE_BYTES: usize = 64 * 1024;
const DEFAULT_MAX_BUFFERED_SEVERITY: Severity = Severity::Info4;
const DEFAULT_OVERFLOW_POLICY: ScopedFlightRecorderOverflowPolicy =
    ScopedFlightRecorderOverflowPolicy::Drop;

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
            .field("overflow_policy", &self.shared.overflow_policy)
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
            overflow_policy: DEFAULT_OVERFLOW_POLICY,
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
    overflow_policy: ScopedFlightRecorderOverflowPolicy,
    max_buffered_severity: Severity,
}

/// Behavior for low-severity records that cannot be retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScopedFlightRecorderOverflowPolicy {
    /// Drops the incoming record, preserving the recorder's ingestion savings.
    Drop,
    /// Sends the incoming record through the wrapped processor.
    Export,
}

/// The reason a new recording scope could not be admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScopedFlightRecorderStartError {
    /// The configured active-scope limit has been reached.
    ScopeLimitReached,
    /// The processor has shut down.
    Shutdown,
    /// The active-scope registry is unavailable.
    InternalFailure,
}

impl fmt::Display for ScopedFlightRecorderStartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScopeLimitReached => f.write_str("flight recorder active-scope limit reached"),
            Self::Shutdown => f.write_str("flight recorder processor is shut down"),
            Self::InternalFailure => f.write_str("flight recorder scope registry is unavailable"),
        }
    }
}

impl Error for ScopedFlightRecorderStartError {}

/// Failure from [`ScopedFlightRecorder::record_on_error`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScopedFlightRecorderOperationError<E> {
    /// A recording scope could not be admitted.
    Start(ScopedFlightRecorderStartError),
    /// The operation failed and its snapshot was handed off successfully.
    Operation(E),
    /// The operation failed and snapshot handoff also failed.
    Handoff {
        /// The original operation error.
        operation: E,
        /// The snapshot handoff error.
        handoff: OTelSdkError,
    },
}

impl<E: fmt::Display> fmt::Display for ScopedFlightRecorderOperationError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start(err) => write!(f, "flight recorder scope admission failed: {err}"),
            Self::Operation(err) => write!(f, "recorded operation failed: {err}"),
            Self::Handoff { operation, handoff } => {
                write!(
                    f,
                    "recorded operation failed ({operation}) and snapshot handoff failed: {handoff}"
                )
            }
        }
    }
}

impl<E: Error + 'static> Error for ScopedFlightRecorderOperationError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Start(err) => Some(err),
            Self::Operation(err) => Some(err),
            Self::Handoff { handoff, .. } => Some(handoff),
        }
    }
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
    /// Larger records are handled by the configured overflow policy.
    pub fn with_max_record_size_bytes(mut self, max_record_size_bytes: usize) -> Self {
        self.max_record_size_bytes = max_record_size_bytes;
        self
    }

    /// Sets how low-severity records are handled when recorder limits prevent
    /// them from being buffered.
    pub fn with_overflow_policy(
        mut self,
        overflow_policy: ScopedFlightRecorderOverflowPolicy,
    ) -> Self {
        self.overflow_policy = overflow_policy;
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
            overflow_policy: self.overflow_policy,
            max_buffered_severity: self.max_buffered_severity,
            metrics: Arc::new(FlightRecorderMetrics::new(
                "scoped_flight_recorder_log_processor",
            )),
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
    /// Returns an explicit error when the configured active-scope limit has
    /// been reached, the processor has shut down, or admission cannot inspect
    /// the active-scope registry.
    pub fn try_start(&self) -> Result<ScopedFlightRecorderScope, ScopedFlightRecorderStartError> {
        self.inner
            .try_start()
            .map(|buffer| ScopedFlightRecorderScope {
                inner: self.inner.clone(),
                buffer,
            })
    }

    /// Runs a fallible operation in a recording scope.
    ///
    /// Successful operations discard their buffered logs. Failed operations
    /// hand off their snapshot without waiting for delegate flush completion.
    /// Cancelling or dropping this future discards the in-progress snapshot.
    /// Use the lower-level scope API when failure handling must call
    /// [`ScopedFlightRecorderScope::trigger`] and wait for a flush.
    pub async fn record_on_error<F, T, E>(
        &self,
        future: F,
    ) -> Result<T, ScopedFlightRecorderOperationError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        let scope = self
            .try_start()
            .map_err(ScopedFlightRecorderOperationError::Start)?;
        match scope.with_context(future).await {
            Ok(value) => {
                scope.discard();
                Ok(value)
            }
            Err(operation) => match scope.handoff() {
                Ok(()) => Err(ScopedFlightRecorderOperationError::Operation(operation)),
                Err(handoff) => {
                    Err(ScopedFlightRecorderOperationError::Handoff { operation, handoff })
                }
            },
        }
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

    /// Hands this scope's buffered snapshot to the wrapped processor and
    /// switches the scope to passthrough without flushing the delegate.
    ///
    /// This avoids waiting for exporter completion, but acquiring recorder
    /// locks and the wrapped processor's `emit` implementation may still block.
    pub fn handoff(&self) -> OTelSdkResult {
        self.inner.handoff(&self.buffer)
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
    fn try_start(&self) -> Result<Arc<ScopeBuffer>, ScopedFlightRecorderStartError>;
    fn handoff(&self, buffer: &Arc<ScopeBuffer>) -> OTelSdkResult;
    fn trigger(&self, buffer: &Arc<ScopeBuffer>) -> OTelSdkResult;
}

struct ScopeBuffer {
    id: usize,
    processor_id: usize,
    max_record_size_bytes: usize,
    memory_budget: Arc<MemoryBudget>,
    metrics: Arc<FlightRecorderMetrics>,
    accounted_bytes: AtomicUsize,
    state: Mutex<ScopeState>,
}

struct ScopeState {
    status: ScopeStatus,
    buffer: LogBuffer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeStatus {
    Recording,
    Passthrough,
    Discarded,
}

enum BufferResult {
    Buffered { evicted: usize },
    Passthrough,
    Overflow,
    Oversized,
}

impl ScopeBuffer {
    fn try_buffer(
        &self,
        record: &SdkLogRecord,
        instrumentation: &InstrumentationScope,
        estimated_size: usize,
    ) -> Result<BufferResult, OTelSdkError> {
        let mut state = self
            .state
            .lock()
            .map_err(|err| mutex_error("scope buffer", err))?;
        if state.status != ScopeStatus::Recording {
            return Ok(BufferResult::Passthrough);
        }
        if estimated_size > self.max_record_size_bytes || !state.buffer.can_fit(estimated_size) {
            return Ok(BufferResult::Oversized);
        }
        let plan = state.buffer.plan_insertion(estimated_size);
        let additional_bytes = estimated_size.saturating_sub(plan.bytes);
        if !self.memory_budget.try_reserve(additional_bytes) {
            return Ok(BufferResult::Overflow);
        }

        if plan.bytes > estimated_size {
            self.memory_budget.release(plan.bytes - estimated_size);
        }
        self.accounted_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |accounted| {
                Some(accounted - plan.bytes + estimated_size)
            })
            .expect("scope byte accounting must not underflow");
        state.buffer.insert(
            (record.clone(), instrumentation.clone(), estimated_size),
            plan,
        );
        Ok(BufferResult::Buffered {
            evicted: plan.count,
        })
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
        self.accounted_bytes.store(0, Ordering::Release);
        Ok(BufferedSnapshot {
            records: state.buffer.take(),
            memory_budget: self.memory_budget.clone(),
        })
    }

    fn discard(&self) {
        match self.state.lock() {
            Ok(mut state) => {
                if state.status != ScopeStatus::Recording {
                    return;
                }
                state.status = ScopeStatus::Discarded;
                state.buffer.clear();
                self.release_all_accounted_bytes();
                self.metrics.scope_discarded();
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
        if self
            .state
            .get_mut()
            .is_ok_and(|state| state.status == ScopeStatus::Recording)
        {
            self.metrics.scope_discarded();
        }
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
    overflow_policy: ScopedFlightRecorderOverflowPolicy,
    max_buffered_severity: Severity,
    metrics: Arc<FlightRecorderMetrics>,
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
            self.metrics.replayed(snapshot.records.len());
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

    fn handoff_snapshot(&self, buffer: Arc<ScopeBuffer>) -> OTelSdkResult {
        let delegate = self
            .delegate
            .write()
            .map_err(|err| lock_error("delegate", err))?;
        let mut snapshot = buffer.take_and_passthrough()?;
        self.metrics.replayed(snapshot.records.len());
        while let Some((mut record, instrumentation, estimated_size)) = snapshot.pop_front() {
            delegate.emit(&mut record, &instrumentation);
            snapshot.release(estimated_size);
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
}

impl<P: LogProcessor + 'static> ScopedFlightRecorderCore for ScopedShared<P> {
    fn try_start(&self) -> Result<Arc<ScopeBuffer>, ScopedFlightRecorderStartError> {
        if self.is_shutdown.load(Ordering::Acquire) {
            self.metrics.scope_rejected();
            return Err(ScopedFlightRecorderStartError::Shutdown);
        }
        let mut scopes = self.scopes.lock().map_err(|_| {
            self.metrics.scope_rejected();
            ScopedFlightRecorderStartError::InternalFailure
        })?;
        scopes.retain(|_, buffer| buffer.strong_count() > 0);
        if self.is_shutdown.load(Ordering::Acquire) {
            self.metrics.scope_rejected();
            return Err(ScopedFlightRecorderStartError::Shutdown);
        }
        if scopes.len() >= self.max_active_scopes {
            self.metrics.scope_rejected();
            return Err(ScopedFlightRecorderStartError::ScopeLimitReached);
        }

        let id = self.next_scope_id.fetch_add(1, Ordering::Relaxed);
        let buffer = Arc::new(ScopeBuffer {
            id,
            processor_id: self.processor_id,
            max_record_size_bytes: self.max_record_size_bytes,
            memory_budget: self.memory_budget.clone(),
            metrics: self.metrics.clone(),
            accounted_bytes: AtomicUsize::new(0),
            state: Mutex::new(ScopeState {
                status: ScopeStatus::Recording,
                buffer: LogBuffer::new(
                    self.max_records_per_scope,
                    self.max_buffer_size_bytes_per_scope,
                ),
            }),
        });
        scopes.insert(id, Arc::downgrade(&buffer));
        self.metrics.scope_admitted();
        Ok(buffer)
    }

    fn handoff(&self, buffer: &Arc<ScopeBuffer>) -> OTelSdkResult {
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
        self.handoff_snapshot(buffer.clone())?;
        self.metrics.handed_off();
        Ok(())
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
        self.flush_snapshots([buffer.clone()])?;
        self.metrics.triggered();
        Ok(())
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

        if should_buffer(record, self.shared.max_buffered_severity) {
            let buffer = Context::map_current(|context| {
                context
                    .get::<ScopedBufferContext>()
                    .map(|context| context.buffer.clone())
            });
            if let Some(buffer) = buffer {
                if buffer.processor_id == self.shared.processor_id {
                    let estimated_size = estimated_log_size(record, instrumentation);
                    match buffer.try_buffer(record, instrumentation, estimated_size) {
                        Ok(BufferResult::Buffered { evicted }) => {
                            self.shared.metrics.buffered(1);
                            self.shared.metrics.evicted(evicted);
                            return;
                        }
                        Ok(BufferResult::Passthrough) => {}
                        Ok(BufferResult::Overflow) => {
                            if self.shared.overflow_policy
                                == ScopedFlightRecorderOverflowPolicy::Export
                            {
                                self.shared.metrics.capacity_overflow(true);
                                self.shared.emit_to_delegate(record, instrumentation);
                            } else {
                                self.shared.metrics.capacity_overflow(false);
                            }
                            return;
                        }
                        Ok(BufferResult::Oversized) => {
                            if self.shared.overflow_policy
                                == ScopedFlightRecorderOverflowPolicy::Export
                            {
                                self.shared.metrics.oversized(true);
                                self.shared.emit_to_delegate(record, instrumentation);
                            } else {
                                self.shared.metrics.oversized(false);
                            }
                            return;
                        }
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
        self.shared.flush_snapshots(buffers)?;
        self.shared.metrics.triggered();
        Ok(())
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        let start = Instant::now();
        let _trigger_guard =
            lock_with_timeout(&self.shared.trigger_lock, timeout).map_err(|err| match err {
                TimedLockError::Poisoned => OTelSdkError::InternalFailure(
                    "ScopedFlightRecorderLogProcessor trigger mutex poisoned".into(),
                ),
                TimedLockError::Timeout => OTelSdkError::Timeout(timeout),
            })?;
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
    fn record_on_error_discards_success_and_hands_off_failure() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let instrumentation = InstrumentationScope::builder("test").build();

        let success = futures_executor::block_on(recorder.record_on_error(async {
            processor.emit(&mut record("success", Severity::Info), &instrumentation);
            Ok::<_, &'static str>("ok")
        }));
        assert_eq!(success.unwrap(), "ok");
        assert!(bodies(&delegate).is_empty());

        let failure = futures_executor::block_on(recorder.record_on_error(async {
            processor.emit(&mut record("failure", Severity::Info), &instrumentation);
            Err::<(), _>("operation failed")
        }));
        assert!(matches!(
            failure,
            Err(ScopedFlightRecorderOperationError::Operation(
                "operation failed"
            ))
        ));
        assert_eq!(bodies(&delegate), ["failure"]);
    }

    #[test]
    fn record_on_error_preserves_start_and_handoff_failures() {
        let delegate = TestProcessor::new();
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate)
            .with_max_active_scopes(1)
            .build();
        let active = recorder.try_start().unwrap();

        let rejected = futures_executor::block_on(
            recorder.record_on_error(async { Ok::<_, &'static str>(()) }),
        );
        assert!(matches!(
            rejected,
            Err(ScopedFlightRecorderOperationError::Start(
                ScopedFlightRecorderStartError::ScopeLimitReached
            ))
        ));
        active.discard();
        drop(active);

        let handoff_failure = futures_executor::block_on(recorder.record_on_error(async {
            processor
                .shutdown_with_timeout(Duration::from_secs(1))
                .unwrap();
            Err::<(), _>("operation failed")
        }));
        match handoff_failure {
            Err(ScopedFlightRecorderOperationError::Handoff {
                operation: "operation failed",
                handoff: OTelSdkError::AlreadyShutdown,
            }) => {}
            other => panic!("unexpected result: {other:?}"),
        }
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
    fn handoff_replays_without_flushing_delegate() {
        let delegate = TestProcessor::new();
        let (processor, recorder) =
            ScopedFlightRecorderLogProcessor::builder(delegate.clone()).build();
        let scope = recorder.try_start().unwrap();
        let instrumentation = InstrumentationScope::builder("test").build();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("before", Severity::Info), &instrumentation);
        }));
        scope.handoff().unwrap();

        assert_eq!(bodies(&delegate), ["before"]);
        assert_eq!(delegate.flushes.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn active_scope_limit_is_enforced_and_reclaimed() {
        let delegate = TestProcessor::new();
        let (_processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate)
            .with_max_active_scopes(1)
            .build();

        let first = recorder.try_start().unwrap();
        assert_eq!(
            recorder.try_start().unwrap_err(),
            ScopedFlightRecorderStartError::ScopeLimitReached
        );
        drop(first);
        assert!(recorder.try_start().is_ok());
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
            .with_overflow_policy(ScopedFlightRecorderOverflowPolicy::Export)
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
    fn aggregate_overflow_drops_by_default() {
        let delegate = TestProcessor::new();
        let instrumentation = InstrumentationScope::builder("test").build();
        let estimated_size = estimated_log_size(&record("first", Severity::Info), &instrumentation);
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
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

        assert!(bodies(&delegate).is_empty());
        first.discard();
        second.discard();
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
            .with_overflow_policy(ScopedFlightRecorderOverflowPolicy::Export)
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
            .with_overflow_policy(ScopedFlightRecorderOverflowPolicy::Export)
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
    fn oversized_scoped_records_drop_by_default() {
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

        assert!(bodies(&delegate).is_empty());
        scope.trigger().unwrap();
        assert!(bodies(&delegate).is_empty());
    }

    #[test]
    fn oversized_records_outside_a_scope_follow_the_normal_path() {
        let delegate = TestProcessor::new();
        let instrumentation = InstrumentationScope::builder("test").build();
        let estimated_size =
            estimated_log_size(&record("oversized", Severity::Info), &instrumentation);
        let (processor, _recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_record_size_bytes(estimated_size - 1)
            .build();

        processor.emit(&mut record("oversized", Severity::Info), &instrumentation);

        assert_eq!(bodies(&delegate), ["oversized"]);
    }

    #[test]
    fn oversized_records_after_trigger_follow_the_normal_path() {
        let delegate = TestProcessor::new();
        let instrumentation = InstrumentationScope::builder("test").build();
        let estimated_size =
            estimated_log_size(&record("oversized", Severity::Info), &instrumentation);
        let (processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate.clone())
            .with_max_record_size_bytes(estimated_size - 1)
            .build();
        let scope = recorder.try_start().unwrap();
        scope.trigger().unwrap();

        futures_executor::block_on(scope.with_context(async {
            processor.emit(&mut record("oversized", Severity::Info), &instrumentation);
        }));

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

        assert_eq!(
            recorder.try_start().unwrap_err(),
            ScopedFlightRecorderStartError::Shutdown
        );
        assert!(matches!(
            scope.trigger(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert!(bodies(&delegate).is_empty());
    }

    #[cfg(feature = "experimental_metrics_bound_instruments")]
    #[test]
    #[ignore]
    fn self_diagnostics_report_scope_admission_rejection_and_discard() {
        use crate::metrics::{InMemoryMetricExporter, SdkMeterProvider};

        let metric_exporter = InMemoryMetricExporter::default();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter.clone())
            .build();
        opentelemetry::global::set_meter_provider(meter_provider.clone());

        let delegate = TestProcessor::new();
        let (_processor, recorder) = ScopedFlightRecorderLogProcessor::builder(delegate)
            .with_max_active_scopes(1)
            .build();
        let scope = recorder.try_start().unwrap();
        assert_eq!(
            recorder.try_start().unwrap_err(),
            ScopedFlightRecorderStartError::ScopeLimitReached
        );
        scope.discard();
        meter_provider.force_flush().unwrap();

        assert_eq!(
            diagnostic_action_total(&metric_exporter, "scope_admitted"),
            1
        );
        assert_eq!(
            diagnostic_action_total(&metric_exporter, "scope_rejected"),
            1
        );
        assert_eq!(
            diagnostic_action_total(&metric_exporter, "scope_discarded"),
            1
        );

        meter_provider.shutdown().unwrap();
    }

    #[cfg(feature = "experimental_metrics_bound_instruments")]
    fn diagnostic_action_total(
        exporter: &crate::metrics::InMemoryMetricExporter,
        action: &str,
    ) -> u64 {
        use crate::metrics::data::{AggregatedMetrics, MetricData};

        exporter
            .get_finished_metrics()
            .unwrap()
            .iter()
            .flat_map(|resource| &resource.scope_metrics)
            .flat_map(|scope| &scope.metrics)
            .filter(|metric| {
                metric
                    .name
                    .starts_with("otel.sdk.processor.log.flight_recorder.")
            })
            .filter_map(|metric| match &metric.data {
                AggregatedMetrics::U64(MetricData::Sum(sum)) => Some(sum),
                _ => None,
            })
            .flat_map(|sum| sum.data_points())
            .filter(|point| {
                point.attributes().any(|attribute| {
                    attribute.key.as_str() == "action" && attribute.value.as_str() == action
                })
            })
            .map(|point| point.value())
            .sum()
    }
}
