use std::time::{SystemTime, UNIX_EPOCH};

const NANOS_PER_SEC: i128 = 1_000_000_000;
const SECS_PER_DAY: i128 = 86_400;

/// Formats `time` in UTC as `YYYY-MM-DD HH:MM:SS.ffffff`, the layout every
/// exporter in this crate prints. The fraction is truncated to microseconds.
///
/// Every `SystemTime` formats: the arithmetic is in `i128`, so no time can
/// overflow it or make it panic.
pub(crate) fn format_time(time: SystemTime) -> String {
    let nanos = match time.duration_since(UNIX_EPOCH) {
        Ok(after) => after.as_nanos() as i128,
        Err(before) => -(before.duration().as_nanos() as i128),
    };
    let secs = nanos.div_euclid(NANOS_PER_SEC);
    let micros = nanos.rem_euclid(NANOS_PER_SEC) / 1_000;
    let (year, month, day) = civil_from_days(secs.div_euclid(SECS_PER_DAY));
    let second_of_day = secs.rem_euclid(SECS_PER_DAY);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}.{micros:06}",
        second_of_day / 3_600,
        second_of_day % 3_600 / 60,
        second_of_day % 60,
    )
}

/// The proleptic Gregorian date `days` after 1970-01-01, by Howard Hinnant's
/// `civil_from_days` (<https://howardhinnant.github.io/date_algorithms.html>).
fn civil_from_days(days: i128) -> (i128, i128, i128) {
    // Count from 0000-03-01, so a leap day is the last day of its year.
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + i128::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::format_time;
    use std::time::{Duration, SystemTime};

    fn at(secs: u64, nanos: u32) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::new(secs, nanos)
    }

    #[test]
    fn formats_utc_with_six_fraction_digits() {
        assert_eq!(format_time(at(0, 0)), "1970-01-01 00:00:00.000000");
        // The fraction is truncated, not rounded.
        assert_eq!(
            format_time(at(1_794_918_896, 123_456_789)),
            "2026-11-17 12:34:56.123456"
        );
    }

    #[test]
    fn formats_leap_days_and_century_years() {
        assert_eq!(
            format_time(at(1_709_208_000, 0)),
            "2024-02-29 12:00:00.000000"
        );
        // 2000 is a leap year; 1900 is not.
        assert_eq!(
            format_time(at(951_865_200, 0)),
            "2000-02-29 23:00:00.000000"
        );
        assert_eq!(
            format_time(SystemTime::UNIX_EPOCH - Duration::from_secs(2_203_891_200)),
            "1900-03-01 00:00:00.000000"
        );
    }

    #[test]
    fn formats_times_before_the_epoch() {
        let time = SystemTime::UNIX_EPOCH - Duration::from_millis(1_500);
        assert_eq!(format_time(time), "1969-12-31 23:59:58.500000");
    }

    #[test]
    fn formats_years_past_9999() {
        assert_eq!(
            format_time(at(253_402_300_799, 999_999_999)),
            "9999-12-31 23:59:59.999999"
        );
        assert_eq!(
            format_time(at(253_402_300_800, 0)),
            "10000-01-01 00:00:00.000000"
        );
    }
}
