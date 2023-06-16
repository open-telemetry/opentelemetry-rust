use core::ffi;
use core::pin;
use eventheader::_internal as ehi;

static METRICS_EVENT: ehi::TracepointState = ehi::TracepointState::new(0);

/// This is the command string for the event. It needs to follow the
/// [Command Format](https://docs.kernel.org/trace/user_events.html#command-format)
/// syntax, it needs to end with a "\0", and it needs to stay in sync with the
/// write function.
///
/// Syntax is: "EventName Field1Type Field1Name;Field2Type Field2Name".
///
/// For this event:
///
/// - Event is named "OpenTelemetryMetrics".
/// - Field 1 is named "data" and has type "variable-length array of u8".
///
/// "__rel_loc" is a special type for variable-length fields. It requires
/// special handling in the write() method.
const METRICS_EVENT_DEF: &[u8] = b"OpenTelemetryMetrics __rel_loc u8[] data\0";

/// If the tracepoint is registered and enabled, writes an event. If the tracepoint
/// is unregistered or disabled, this does nothing and returns 0. You should usually
/// check [`enabled()`] and only build the data and call `write()` if `enabled()`
/// returns true.
///
/// Requires: data.len() < 65536.
///
/// Return value is 0 for success or an errno code for error. The return value is
/// provided to help with debugging and should usually be ignored in release builds.
pub fn write(data: &[u8]) -> i32 {
    // This must stay in sync with the METRICS_EVENT_DEF string.
    // Return error -1 if data exceeds max size
    if data.len() > u16::MAX as usize {
        println!("Data exceeds max length.");
        return -1;
    }

    // The rel_loc for the data field stores the size and offset of the data.
    // - High 16 bits store the size = data.len()
    // - Low 16 bits store the offset of the data from the end of the rel_loc field = 0.
    let data_rel_loc: u32 = (data.len() as u32) << 16;

    METRICS_EVENT.write(&mut [
        // mut because the write method does some fix-ups.
        ehi::EventDataDescriptor::zero(), // First item in array MUST be zero().
        ehi::EventDataDescriptor::from_value(&data_rel_loc), // rel_loc for the data field.
        ehi::EventDataDescriptor::from_slice(data), // data field.
    ])
}

/// Returns true if this tracepoint is registered and enabled.
#[inline(always)]
pub fn enabled() -> bool {
    METRICS_EVENT.enabled()
}

/// Registers this tracepoint.
///
/// Requires: this tracepoint is not currently registered.
///
/// Return value is 0 for success or an errno code for error. The return value is
/// provided to help with debugging and should usually be ignored in release builds.
///
/// # Safety
///
/// If this code is used in a shared object (DLL), the tracepoint MUST be
/// unregistered before the shared object unloads from memory.
pub unsafe fn register() -> i32 {
    debug_assert!(METRICS_EVENT_DEF[METRICS_EVENT_DEF.len() - 1] == b'\0');

    // Pin is OK because METRICS_EVENT is static.
    // CStr::from_bytes_with_nul_unchecked is ok because METRICS_EVENT_DEF ends with "\0".
    pin::Pin::new_unchecked(&METRICS_EVENT)
        .register(ffi::CStr::from_bytes_with_nul_unchecked(METRICS_EVENT_DEF))
}

/// Unregisters this tracepoint.
///
/// Return value is 0 for success or an errno code for error. The return value is
/// provided to help with debugging and should usually be ignored in release builds.
pub fn unregister() -> i32 {
    METRICS_EVENT.unregister()
}
