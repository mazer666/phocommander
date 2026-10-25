//! The formats of the values that the interface writes.
//!
//! WHAT: the seam that carries the formats of a locale ([`Formats`]) and the
//!       formats that carry no rule of a locale ([`PlainFormats`]).
//! WHY: the numbers, the sizes, the dates and the times of the interface
//!       follow the rules of the locale of the user (FR-123). Those rules
//!       live in the data of CLDR, and the interface crate may not depend on
//!       the crate that owns that data — the crate rule of ADR-0001 forbids a
//!       dependency on a sibling. The seam is therefore a trait, and the
//!       composition root hands the implementation over, exactly as it does
//!       for the texts of the catalogue.
//! BOUNDARY: this module writes the values of the interface. It reads no
//!       file, it asks no operating system for a locale and it knows nothing
//!       about widgets or panels.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::format::{Formats, PlainFormats};
//!
//! let formats = PlainFormats;
//! assert_eq!(formats.size(2048), "2.0 KiB");
//! ```

use std::time::SystemTime;

/// The formats of the values that the interface writes.
///
/// WHY: the surface must not know how a locale writes its numbers, its sizes
///       and its dates — that is the business of the localisation module. The
///       trait is the seam between the two, and the shell that owns the
///       catalogue of the texts owns the formats as well: a language is not
///       only the texts, it is also the way the numbers and the dates are
///       written.
pub trait Formats {
    /// A whole number, written the way the locale writes numbers.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::format::{Formats, PlainFormats};
    ///
    /// assert_eq!(PlainFormats.number(1234), "1234");
    /// ```
    fn number(&self, value: u64) -> String;

    /// A size in bytes, written the way the locale writes sizes.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::format::{Formats, PlainFormats};
    ///
    /// assert_eq!(PlainFormats.size(512), "512 B");
    /// ```
    fn size(&self, bytes: u64) -> String;

    /// The date of a moment, written the way the locale writes dates.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::format::{Formats, PlainFormats};
    /// use std::time::{Duration, UNIX_EPOCH};
    ///
    /// let moment = UNIX_EPOCH + Duration::from_secs(1_715_956_200);
    /// assert_eq!(PlainFormats.date(moment), "2024-05-17");
    /// ```
    fn date(&self, moment: SystemTime) -> String;

    /// The time of the day of a moment, written the way the locale writes
    /// times.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::format::{Formats, PlainFormats};
    /// use std::time::{Duration, UNIX_EPOCH};
    ///
    /// let moment = UNIX_EPOCH + Duration::from_secs(1_715_956_200);
    /// assert_eq!(PlainFormats.time(moment), "14:30");
    /// ```
    fn time(&self, moment: SystemTime) -> String;

    /// The date and the time of a moment, written the way the locale writes
    /// them.
    ///
    /// WHY: the column of the date of a file list carries the date and the
    ///       time together. A locale may write them with a separator of its
    ///       own, so the implementation of a locale may replace this answer.
    ///       The plain formats join the two parts with a space.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::format::{Formats, PlainFormats};
    /// use std::time::{Duration, UNIX_EPOCH};
    ///
    /// let moment = UNIX_EPOCH + Duration::from_secs(1_715_956_200);
    /// assert_eq!(PlainFormats.date_time(moment), "2024-05-17 14:30");
    /// ```
    fn date_time(&self, moment: SystemTime) -> String {
        format!("{} {}", self.date(moment), self.time(moment))
    }
}

/// The formats that carry no rule of a locale.
///
/// WHY: a caller that has no locale — a test, a tool that reads a listing —
///       still needs sizes and dates. These formats are the ones the product
///       wrote before the locales were part of it: the sizes in the binary
///       units, the date as `YYYY-MM-DD` and the time as `HH:MM`. They are
///       the fallback of the seam and not the format of the product: the
///       composition root installs the formats of the locale of the user.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainFormats;

impl Formats for PlainFormats {
    fn number(&self, value: u64) -> String {
        value.to_string()
    }

    fn size(&self, bytes: u64) -> String {
        let mut value = bytes as f64;
        let mut unit = 0;
        while value >= SIZE_STEP && unit + 1 < SIZE_UNITS.len() {
            value /= SIZE_STEP;
            unit += 1;
        }

        if unit == 0 {
            format!("{bytes} {}", SIZE_UNITS[0])
        } else {
            format!("{value:.1} {}", SIZE_UNITS[unit])
        }
    }

    fn date(&self, moment: SystemTime) -> String {
        let parts = Parts::of(moment);
        format!("{:04}-{:02}-{:02}", parts.year, parts.month, parts.day)
    }

    fn time(&self, moment: SystemTime) -> String {
        let parts = Parts::of(moment);
        format!("{:02}:{:02}", parts.hour, parts.minute)
    }
}

/// The units of a size, in the order in which they grow.
///
/// WHY: the units are written with the prefix of the binary system, because
///       these are the formats the product wrote before the locales were
///       part of it. The formats of a locale use the units of the
///       international system and the separator of the locale.
const SIZE_UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

/// The factor between two units of a size.
const SIZE_STEP: f64 = 1024.0;

/// The parts of a moment in the civil calendar.
struct Parts {
    /// The year.
    year: i32,
    /// The month, from 1 to 12.
    month: u8,
    /// The day of the month, from 1 to 31.
    day: u8,
    /// The hour, from 0 to 23.
    hour: u8,
    /// The minute, from 0 to 59.
    minute: u8,
}

impl Parts {
    /// The parts of a moment.
    ///
    /// WHY: the conversion from the day number to a date is the civil
    ///       calendar of Howard Hinnant, which needs no table and no library.
    ///       The moment is read in the time zone of UTC, because the platform
    ///       layer has not yet supplied the offset of the time zone of the
    ///       machine.
    fn of(moment: SystemTime) -> Parts {
        let seconds = match moment.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() as i64,
            Err(error) => -(error.duration().as_secs() as i64),
        };

        let days = seconds.div_euclid(86_400);
        let time_of_day = seconds.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);

        Parts {
            year: year as i32,
            month: month as u8,
            day: day as u8,
            hour: (time_of_day / 3_600) as u8,
            minute: ((time_of_day % 3_600) / 60) as u8,
        }
    }
}

/// Converts a number of days since 1970-01-01 into a civil date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::time::Duration;

    /// The moment of 2024-05-17, 14:30:00 in the time zone of UTC.
    fn moment() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_715_956_200)
    }

    #[test]
    fn writes_a_size_the_way_a_file_manager_wrote_it() {
        // act
        let sizes = [
            PlainFormats.size(0),
            PlainFormats.size(512),
            PlainFormats.size(2_048),
            PlainFormats.size(5_368_709_120),
        ];

        // assert
        assert_eq!(sizes[0], "0 B");
        assert_eq!(sizes[1], "512 B");
        assert_eq!(sizes[2], "2.0 KiB");
        assert_eq!(sizes[3], "5.0 GiB");
    }

    #[test]
    fn writes_a_date_and_a_time_the_way_the_product_wrote_it() {
        // act
        let date = PlainFormats.date(moment());
        let time = PlainFormats.time(moment());
        let both = PlainFormats.date_time(moment());

        // assert
        assert_eq!(date, "2024-05-17");
        assert_eq!(time, "14:30");
        assert_eq!(both, "2024-05-17 14:30");
    }

    #[test]
    fn writes_a_number_without_a_separator() {
        // act
        let text = PlainFormats.number(1_234_567);

        // assert
        assert_eq!(text, "1234567");
    }
}
