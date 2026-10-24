//! The formatting of a point in time.
//!
//! WHAT: the date of a file ([{format_date}]) and the time of a file
//!       ([{format_time}]).
//! WHY: the rename placeholders {[date]} and {[time]} have to show a date
//!       that is the same on every platform and in every language, because
//!       the name of a file is not translated. The format is the one of the
//!       specification: the date as 'YYYY-MM-DD' and the time as 'HH-MM-SS'.
//!       The dashes instead of the colons keep the name usable on every
//!       platform.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::datetime::format_date;
//! use std::time::{Duration, SystemTime};
//!
//! let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
//! assert_eq!(format_date(time), "2023-11-14");
//! ```

use std::time::{Duration, SystemTime};

/// The number of seconds between the epoch and the first of January 1970 in
/// the time zone of the file system.
const SECONDS_PER_DAY: i64 = 86_400;

/// Formats the date of a point in time as 'YYYY-MM-DD'.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::datetime::format_date;
/// use std::time::{Duration, SystemTime};
///
/// let time = SystemTime::UNIX_EPOCH + Duration::from_secs(0);
/// assert_eq!(format_date(time), "1970-01-01");
/// ```
pub fn format_date(time: SystemTime) -> String {
    let (year, month, day) = civil_from_days(days_since_epoch(time));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Formats the time of a point in time as 'HH-MM-SS'.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::datetime::format_time;
/// use std::time::{Duration, SystemTime};
///
/// let time = SystemTime::UNIX_EPOCH + Duration::from_secs(3661);
/// assert_eq!(format_time(time), "01-01-01");
/// ```
pub fn format_time(time: SystemTime) -> String {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let day_seconds = (seconds % SECONDS_PER_DAY as u64) as i64;
    let hours = day_seconds / 3600;
    let minutes = (day_seconds % 3600) / 60;
    let rest = day_seconds % 60;
    format!("{hours:02}-{minutes:02}-{rest:02}")
}

/// The number of days between the epoch and the point in time.
fn days_since_epoch(time: SystemTime) -> i64 {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64;
    seconds.div_euclid(SECONDS_PER_DAY)
}

/// Converts a number of days since the epoch into a year, a month and a day.
///
/// WHY: the conversion follows the calendar of the proleptic Gregorian
/// calendar, so that the date of a file does not depend on the time zone of
/// the machine that renames it.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 { shifted } else { shifted - 146_096 } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_position + 2) / 5 + 1) as u32;
    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    } as u32;

    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_is_the_first_of_january_1970() {
        // act
        let date = format_date(SystemTime::UNIX_EPOCH);

        // assert
        assert_eq!(date, "1970-01-01");
    }

    #[test]
    fn a_known_moment_has_a_known_date() {
        // arrange
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

        // act
        let date = format_date(time);

        // assert
        assert_eq!(date, "2023-11-14");
    }

    #[test]
    fn a_leap_day_is_formatted_correctly() {
        // arrange
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_709_164_800);

        // act
        let date = format_date(time);

        // assert
        assert_eq!(date, "2024-02-29");
    }

    #[test]
    fn the_time_is_formatted_with_two_digits() {
        // arrange
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(3661);

        // act
        let formatted = format_time(time);

        // assert
        assert_eq!(formatted, "01-01-01");
    }
}
