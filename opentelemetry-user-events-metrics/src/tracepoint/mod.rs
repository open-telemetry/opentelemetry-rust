use core::ffi;
use eventheader::_internal as ehi;
use std::pin::Pin;

/// This is the command string for the event. It needs to follow the
/// [Command Format](https://docs.kernel.org/trace/user_events.html#command-format)
/// syntax, it needs to end with a "\0", and it needs to stay in sync with the
/// write function.
///
/// Syntax is: "EventName Field1Type Field1Name;Field2Type Field2Name".
///
/// For this event:
///
/// - Event is named "otlp_metrics".
/// - Field 1 is named "buffer" and has type "variable-length array of u8".
///
/// "__rel_loc" is a special type for variable-length fields. It requires
/// special handling in the write() method.
const METRICS_EVENT_DEF: &[u8] = b"otlp_metrics __rel_loc u8[] buffer\0";

/// If the tracepoint is registered and enabled, writes an event. If the tracepoint
/// is unregistered or disabled, this does nothing and returns 0. You should usually
/// check [`enabled()`] and only build the buffer and call `write()` if `enabled()`
/// returns true.
///
/// Requires: buffer.len() < 65536.
///
/// Return value is 0 for success or an errno code for error. The return value is
/// provided to help with debugging and should usually be ignored in release builds.
pub fn write(trace_point: &ehi::TracepointState, buffer: &[u8]) -> i32 {
    // This must stay in sync with the METRICS_EVENT_DEF string.
    // Return error -1 if buffer exceeds max size
    if buffer.len() > u16::MAX as usize {
        eprintln!("Buffer exceeds max length.");
        return -1;
    }

    // The rel_loc for the buffer field stores the size and offset of the buffer.
    // - High 16 bits store the size = buffer.len()
    // - Low 16 bits store the offset of the buffer from the end of the rel_loc field = 0.
    let buffer_rel_loc: u32 = (buffer.len() as u32) << 16;

    trace_point.write(&mut [
        // mut because the write method does some fix-ups.
        ehi::EventDataDescriptor::zero(), // First item in array MUST be zero().
        ehi::EventDataDescriptor::from_value(&buffer_rel_loc), // rel_loc for the buffer field.
        ehi::EventDataDescriptor::from_slice(buffer), // buffer field.
    ])
}

/// Registers the passed in tracepoint.
///
/// Requires: this tracepoint is not currently registered.
/// The tracepoint must be in a Pin<&TracepointState> because we must ensure it will never be moved
///
/// Return value is 0 for success or an errno code for error. The return value is
/// provided to help with debugging and should usually be ignored in release builds.
///
/// # Safety
///
/// If this code is used in a shared object, the tracepoint MUST be
/// unregistered before the shared object unloads from memory.
pub unsafe fn register(trace_point: Pin<&ehi::TracepointState>) -> i32 {
    debug_assert!(METRICS_EVENT_DEF[METRICS_EVENT_DEF.len() - 1] == b'\0');

    // CStr::from_bytes_with_nul_unchecked is ok because METRICS_EVENT_DEF ends with "\0".
    trace_point.register(ffi::CStr::from_bytes_with_nul_unchecked(METRICS_EVENT_DEF))
}
