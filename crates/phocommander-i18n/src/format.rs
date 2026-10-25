// The formats of a locale: numbers, sizes, dates and times.
//
// WHAT: the formatter of a locale ([`LocaleFormat`]) and the reasons why it
//       cannot be built ([`FormatError`]).
// WHY: the numbers, the sizes, the dates and the times of the interface follow
//       the locale of the system (FR-123). The rules of a locale — the
//       separator of the decimals, the order of the parts of a date, the
//       cycle of the clock — live in the data of CLDR, and the crate `icu`
//       reads them. The application therefore carries no table of separators
//       of its own, and a language that is added needs no new rule either.
// BOUNDARY: this module formats values. It reads no file, it asks no
//       operating system and it knows nothing about the interface.
//
// EXAMPLE:
// ```no_run
// use phocommander_i18n::LocaleFormat;
//
// # fn main() -> Result<(), phocommander_i18n::FormatError> {
// let german = LocaleFormat::for_tag("de-DE")?;
// assert_eq!(german.decimal(1.2, 1), "1,2");
// # Ok(())
// # }
// ```

use std::time::SystemTime;

use icu::calendar::Date;
use icu::datetime::fieldsets::{T, YMD};
use icu::datetime::input::Time;
use icu::datetime::{DateTimeFormatter, DateTimeFormatterPreferences, NoCalendarFormatter};
use icu::decimal::input::Decimal;
use icu::decimal::options::DecimalFormatterOptions;
use icu::decimal::{DecimalFormatter, DecimalFormatterPreferences};
use icu::locale::Locale as IcuLocale;
use writeable::Writeable;

/// The units of a size, in the order in which they grow.
///
/// WHY: the units are written with the prefix of the international system, so
/// that a size of 1 200 000 bytes is "1.2 MB" and not "1.2 MiB". The choice
/// between the decimal and the binary prefixes belongs to the setting
/// `file_size_format` of the configuration file.
const SIZE_UNITS: [&str; 6] = ["B", "kB", "MB", "GB", "TB", "PB"];

/// The factor between two units of a size.
const SIZE_STEP: f64 = 1000.0;

/// The formats of a locale.
#[derive(Debug, Clone)]
pub struct LocaleFormat {
    /// The tag of the locale, for example `de-CH`.
    tag: String,

    /// The formatter of the decimal numbers.
    decimals: DecimalFormatter,

    /// The formatter of the dates.
    dates: DateTimeFormatter<YMD>,

    /// The formatter of the times.
    times: NoCalendarFormatter<T>,
}

impl LocaleFormat {
    /// Builds the formats of a locale.
    ///
    /// WHY: the tag is a BCP 47 language tag. A tag that the data of CLDR does
    /// not know is an error and not an empty text, because a number that is
    /// written without a separator is a defect that the user cannot see.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::LocaleFormat;
    ///
    /// # fn main() -> Result<(), phocommander_i18n::FormatError> {
    /// let english = LocaleFormat::for_tag("en-US")?;
    /// assert_eq!(english.number(1234), "1,234");
    /// # Ok(())
    /// # }
    /// ```
    pub fn for_tag(tag: &str) -> Result<Self, FormatError> {
        let locale = IcuLocale::try_from_str(tag).map_err(|_| FormatError::UnknownLocale {
            locale: tag.to_string(),
        })?;

        let decimal_preferences = DecimalFormatterPreferences::from(&locale);
        let decimals =
            DecimalFormatter::try_new(decimal_preferences, DecimalFormatterOptions::default())
                .map_err(|error| FormatError::Unavailable {
                    locale: tag.to_string(),
                    reason: error.to_string(),
                })?;

        let datetime_preferences = DateTimeFormatterPreferences::from(&locale);
        let dates =
            DateTimeFormatter::try_new(datetime_preferences, YMD::medium()).map_err(|error| {
                FormatError::Unavailable {
                    locale: tag.to_string(),
                    reason: error.to_string(),
                }
            })?;
        let times =
            NoCalendarFormatter::try_new(datetime_preferences, T::hm()).map_err(|error| {
                FormatError::Unavailable {
                    locale: tag.to_string(),
                    reason: error.to_string(),
                }
            })?;

        Ok(Self {
            tag: tag.to_string(),
            decimals,
            dates,
            times,
        })
    }

    /// The formats of the locale the operating system asks for.
    ///
    /// WHY: the formats follow the locale of the system (FR-123). A system
    /// that reports no locale at all falls back to English, which is the
    /// fallback of the whole product.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::LocaleFormat;
    ///
    /// # fn main() -> Result<(), phocommander_i18n::FormatError> {
    /// let format = LocaleFormat::for_system()?;
    /// assert!(!format.tag().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn for_system() -> Result<Self, FormatError> {
        let tag = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
        LocaleFormat::for_tag(&tag)
    }

    /// The tag of the locale.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// A whole number, written the way the locale writes numbers.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::LocaleFormat;
    ///
    /// # fn main() -> Result<(), phocommander_i18n::FormatError> {
    /// let german = LocaleFormat::for_tag("de-DE")?;
    /// assert_eq!(german.number(1234), "1.234");
    /// # Ok(())
    /// # }
    /// ```
    pub fn number(&self, value: u64) -> String {
        let decimal = Decimal::from(value);
        self.decimals.format_to_string(&decimal)
    }

    /// A number with decimal places, written the way the locale writes them.
    ///
    /// WHY: the separator of the decimals is a rule of the locale. German
    /// writes "1,2", English writes "1.2", and the rule lives in the data of
    /// CLDR and not in the source code.
    pub fn decimal(&self, value: f64, places: u8) -> String {
        let factor = 10_f64.powi(i32::from(places));
        let scaled = (value * factor).round();
        let mut decimal = Decimal::from(scaled as i64);
        decimal.multiply_pow10(-i16::from(places));
        self.decimals.format_to_string(&decimal)
    }

    /// The size of a file, written the way the locale writes sizes.
    ///
    /// WHY: a size is a number and a unit, and the number follows the rules of
    /// the locale. The size of 1 200 000 bytes is therefore "1,2 MB" in German
    /// and "1.2 MB" in English — one format, two languages.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::LocaleFormat;
    ///
    /// # fn main() -> Result<(), phocommander_i18n::FormatError> {
    /// let german = LocaleFormat::for_tag("de-DE")?;
    /// assert_eq!(german.size(1_200_000), "1,2 MB");
    /// # Ok(())
    /// # }
    /// ```
    pub fn size(&self, bytes: u64) -> String {
        let mut value = bytes as f64;
        let mut unit = 0;
        while value >= SIZE_STEP && unit + 1 < SIZE_UNITS.len() {
            value /= SIZE_STEP;
            unit += 1;
        }

        if unit == 0 {
            return format!("{} {}", self.number(bytes), SIZE_UNITS[0]);
        }
        format!("{} {}", self.decimal(value, 1), SIZE_UNITS[unit])
    }

    /// The date of a moment, written the way the locale writes dates.
    ///
    /// WHY: the order of the parts of a date is a rule of the locale. German
    /// writes the day before the month, English writes the month before the
    /// day.
    pub fn date(&self, moment: SystemTime) -> String {
        let parts = Parts::of(moment);
        match Date::try_new_iso(parts.year, parts.month, parts.day) {
            Ok(date) => self.dates.format(&date).write_to_string().into_owned(),
            Err(_) => String::new(),
        }
    }

    /// The time of the day of a moment, written the way the locale writes
    /// times.
    ///
    /// WHY: the cycle of the clock is a rule of the locale. German writes
    /// "14:30", English writes "2:30 PM".
    pub fn time(&self, moment: SystemTime) -> String {
        let parts = Parts::of(moment);
        match Time::try_new(parts.hour, parts.minute, 0, 0) {
            Ok(time) => self.times.format(&time).write_to_string().into_owned(),
            Err(_) => String::new(),
        }
    }

    /// The date and the time of a moment, written the way the locale writes
    /// them.
    pub fn date_time(&self, moment: SystemTime) -> String {
        format!("{} {}", self.date(moment), self.time(moment))
    }
}

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
    /// WHY: the conversion from the day number to a date is the civil calendar
    /// of Howard Hinnant, which needs no table and no library. The moment is
    /// read in the time zone of UTC, because the platform layer has not yet
    /// supplied the offset of the time zone of the machine.
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

/// The reasons why the formats of a locale could not be built.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FormatError {
    /// The tag is not a language tag.
    #[error("the locale {locale} is not a language tag")]
    UnknownLocale {
        /// The tag that was asked for.
        locale: String,
    },

    /// The data of the locale does not carry the format.
    #[error("the locale {locale} carries no data for the format: {reason}")]
    Unavailable {
        /// The tag that was asked for.
        locale: String,

        /// The reason the data reported.
        reason: String,
    },
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    /// The formatter of a tag, or a panic with the reason.
    fn format(tag: &str) -> LocaleFormat {
        match LocaleFormat::for_tag(tag) {
            Ok(format) => format,
            Err(error) => panic!("the formats of {tag} should be readable: {error}"),
        }
    }

    /// The moment of 2024-05-17, 14:30:00 in the time zone of UTC.
    ///
    /// WHY: the formatter reads the moment in the time zone of UTC, because
    ///       the platform layer has not yet supplied the offset of the time
    ///       zone of the machine. The moment of the test is therefore the
    ///       one that carries 14:30 in that time zone.
    fn moment() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_715_956_200)
    }

    #[test]
    fn a_whole_number_follows_the_locale() {
        // act
        let english = format("en-US").number(1234);
        let german = format("de-DE").number(1234);

        // assert
        assert_eq!(english, "1,234");
        assert_eq!(german, "1.234");
    }

    #[test]
    fn a_decimal_number_follows_the_locale() {
        // act
        let english = format("en-US").decimal(1.2, 1);
        let german = format("de-DE").decimal(1.2, 1);

        // assert
        assert_eq!(english, "1.2");
        assert_eq!(german, "1,2");
    }

    #[test]
    fn a_size_is_one_format_that_follows_the_locale() {
        // act
        let english = format("en-US").size(1_200_000);
        let german = format("de-DE").size(1_200_000);

        // assert
        assert_eq!(english, "1.2 MB");
        assert_eq!(german, "1,2 MB");
    }

    #[test]
    fn a_size_in_bytes_is_a_whole_number() {
        // act
        let german = format("de-DE").size(512);

        // assert
        assert_eq!(german, "512 B");
    }

    #[test]
    fn a_date_follows_the_order_of_the_locale() {
        // act
        let english = format("en-US").date(moment());
        let german = format("de-DE").date(moment());

        // assert
        // WHY: English writes the month before the day, German writes the day
        //       before the month. The test reads the order out of the text
        //       and not out of a fixed shape, because the shape of the
        //       medium date is a rule of the data of CLDR and not a rule of
        //       this crate.
        assert!(english.starts_with("May"), "{english}");
        assert!(english.contains("17"), "{english}");
        assert!(english.contains("2024"), "{english}");
        assert!(german.starts_with("17"), "{german}");
        assert!(german.contains("05.2024"), "{german}");
    }

    #[test]
    fn the_time_follows_the_cycle_of_the_clock_of_the_locale() {
        // act
        let english = format("en-US").time(moment());
        let german = format("de-DE").time(moment());

        // assert
        assert!(english.contains("PM"), "{english}");
        assert_eq!(german, "14:30");
    }

    #[test]
    fn an_unknown_tag_is_refused() {
        // act
        let result = LocaleFormat::for_tag("not a tag");

        // assert
        assert!(matches!(result, Err(FormatError::UnknownLocale { .. })));
    }
}
