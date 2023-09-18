use core::ffi;
use eventheader::_internal as ehi;
use std::panic;
use std::pin::Pin;

/// Protocol constant
const PROTOCOL_FIELD_VALUE: u32 = 0;
/// Protobuf definition version
const PROTOBUF_VERSION: &[u8; 8] = b"v0.19.00";

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
/// - Field 1 is named "protocol". Value 0 corresponds to protobuf.
/// - Field 2 is named "version". Corresponds to protocol version (protobuf version).
/// - Field 3 is named "buffer" and has type "variable-length array of u8".
///
/// "__rel_loc" is a special type for variable-length fields. It requires
/// special handling in the write() method.
const METRICS_EVENT_DEF: &[u8] =
    b"otlp_metrics u32 protocol;char[8] version;__rel_loc u8[] buffer;\0";

/// If the tracepoint is registered and enabled, writes an event. If the tracepoint
/// is unregistered or disabled, this does nothing and returns 0. You should usually
/// check [`enabled()`] and only build the buffer and call `write()` if `enabled()`
/// returns true.
///
/// Requires: PROTOBUF_VERSION.len() == 8, buffer.len() < 65536.
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

    if PROTOBUF_VERSION.len() != 8 {
        eprintln!("Version must be char[8].");
        return -1;
    }

    // The rel_loc for the buffer field stores the size and offset of the buffer.
    // - High 16 bits store the size = buffer.len()
    // - Low 16 bits store the offset of the buffer from the end of the rel_loc field = 0.
    let buffer_rel_loc: u32 = (buffer.len() as u32) << 16;

    trace_point.write(&mut [
        // mut because the write method does some fix-ups.
        ehi::EventDataDescriptor::zero(), // First item before buffer MUST be zero().
        ehi::EventDataDescriptor::from_value(&PROTOCOL_FIELD_VALUE), // protocol value 0 for protobuf
        ehi::EventDataDescriptor::from_slice(PROTOBUF_VERSION),      // protobuf definition version
        ehi::EventDataDescriptor::from_value(&buffer_rel_loc), // rel_loc for the buffer field.
        ehi::EventDataDescriptor::from_slice(buffer),          // buffer field.
    ])
}

/// Registers the passed in tracepoint.
///
/// Requires: this tracepoint is not currently registered.
/// The tracepoint must be in a Pin<&TracepointState> because we must ensure it will never be moved
///
/// Return value is 0 for success or -1 for failed register.
///
/// # Safety
///
/// If this code is used in a shared object, the tracepoint MUST be
/// unregistered before the shared object unloads from memory.
pub unsafe fn register(trace_point: Pin<&ehi::TracepointState>) -> i32 {
    debug_assert!(METRICS_EVENT_DEF[METRICS_EVENT_DEF.len() - 1] == b'\0');

    // CStr::from_bytes_with_nul_unchecked is ok because METRICS_EVENT_DEF ends with "\0".
    // Returns errno code 95 if trace/debug file systems are not mounted
    // Returns errno code 13 if insufficient permissions
    // If tracepoint doesn't exist, it will create one automatically
    let result = panic::catch_unwind(|| {
        // CStr::from_bytes_with_nul_unchecked is ok because METRICS_EVENT_DEF ends with "\0".
        trace_point.register(ffi::CStr::from_bytes_with_nul_unchecked(METRICS_EVENT_DEF))
    });

    match result {
        Ok(value) => {
            if value == 0 {
                0
            } else {
                if value == 95 {
                    eprintln!("Trace/debug file systems are not mounted.");
                } else if value == 13 {
                    eprintln!("Insuufficient permissions. You need read/write/execute permissions to user_events tracing directory.");
                }
                -1
            }
        }
        // We don't want to ever panic so we catch the error and return a unique code for retry
        Err(_err) => {
            eprintln!("Tracepoint failed to register. Try restarting your application.");
            -1
        }
    }
}
