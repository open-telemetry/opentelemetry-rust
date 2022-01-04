use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Return the numeric value corresponding to the time for the specified date
/// according to universal time. The value returned is the number of milliseconds
/// since 1 January 1970 00:00:00.
pub(crate) fn get_time(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn test_get_time() {
        assert_eq!(0, get_time(UNIX_EPOCH));
        assert_eq!(0, get_time(UNIX_EPOCH + Duration::from_nanos(1)));
        assert_eq!(1, get_time(UNIX_EPOCH + Duration::from_millis(1)));
        assert_eq!(1000, get_time(UNIX_EPOCH + Duration::from_secs(1)));
    }
}
