//! The placeholders of a rename format.
//!
//! WHAT: the placeholder ([{Placeholder}]), the values that the placeholders
//!       are replaced with ([{Values}]) and the evaluation of a format
//!       ([{evaluate}]).
//! WHY: the placeholders are the building blocks of a rename rule (FR-30):
//!       the name of the file, its extension, the name of its directory, the
//!       number of the counter, the date, the time, the size, the short
//!       checksum, the value that stands for a random number and the four
//!       ways to write a text in a different case. A placeholder that is not
//!       known stays in the name as it is, so that a name that carries a
//!       square bracket of its own is not damaged.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::template::Placeholder;
//!
//! assert_eq!(Placeholder::parse("name"), Some(Placeholder::Name));
//! assert_eq!(Placeholder::parse("unknown"), None);
//! ```
//!
//! BOUNDARY: this module knows the *names* of the placeholders and the values
//!       that they carry. It reads no file and draws nothing.

use std::time::SystemTime;

use crate::template::series::SeriesValues;
use crate::template::{
    RenameInput, TemplateError, base_name_of, extension_of, file_name_of, parent_name_of,
};

/// The character that stands for the counter between the phases.
///
/// WHY: the counter is written after the search and the replacement, so that
///       a replacement rule cannot change a number that the user wants to
///       keep. The character cannot occur in a name of a file, and it is
///       removed before the name is handed back.
pub(crate) const COUNTER_MARKER: char = '\u{0}';

/// One placeholder of a format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placeholder {
    /// The name of the file without its extension.
    Name,

    /// The extension of the file without the dot.
    Extension,

    /// The name of the directory the file lives in.
    Parent,

    /// The number of the counter.
    Counter,

    /// The date of the rename.
    Date,

    /// The time of the rename.
    Time,

    /// The size of the file in bytes.
    Size,

    /// The first eight characters of the checksum of the file.
    Hash,

    /// A value that is derived from the name of the file.
    Random,

    /// The name of the file in upper case.
    Upper,

    /// The name of the file in lower case.
    Lower,

    /// The name of the file with its first letter in upper case.
    Capitalised,

    /// The name of the file with every word in title case.
    Title,

    /// The name of the series.
    Series,

    /// The number of the season.
    Season,

    /// The number of the episode.
    Episode,

    /// The title of the episode — the name of the file without the pattern of
    /// the season and the episode.
    EpisodeTitle,
}

impl Placeholder {
    /// Every placeholder of the dialog.
    ///
    /// WHY: the dialog offers the placeholders in a list, and the list has to
    ///       be the same list everywhere.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::Placeholder;
    ///
    /// assert_eq!(Placeholder::ALL.len(), 17);
    /// ```
    pub const ALL: [Placeholder; 17] = [
        Placeholder::Name,
        Placeholder::Extension,
        Placeholder::Parent,
        Placeholder::Counter,
        Placeholder::Date,
        Placeholder::Time,
        Placeholder::Size,
        Placeholder::Hash,
        Placeholder::Random,
        Placeholder::Upper,
        Placeholder::Lower,
        Placeholder::Capitalised,
        Placeholder::Title,
        Placeholder::Series,
        Placeholder::Season,
        Placeholder::Episode,
        Placeholder::EpisodeTitle,
    ];

    /// Reads the placeholder of a name.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::Placeholder;
    ///
    /// assert_eq!(Placeholder::parse("ext"), Some(Placeholder::Extension));
    /// assert_eq!(Placeholder::parse("name"), None);
    /// ```
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "name" => Some(Placeholder::Name),
            "ext" => Some(Placeholder::Extension),
            "parent" => Some(Placeholder::Parent),
            "n" => Some(Placeholder::Counter),
            "date" => Some(Placeholder::Date),
            "time" => Some(Placeholder::Time),
            "size" => Some(Placeholder::Size),
            "hash" => Some(Placeholder::Hash),
            "rand" => Some(Placeholder::Random),
            "upper" => Some(Placeholder::Upper),
            "lower" => Some(Placeholder::Lower),
            "case" => Some(Placeholder::Capitalised),
            "title" => Some(Placeholder::Title),
            "series" => Some(Placeholder::Series),
            "season" => Some(Placeholder::Season),
            "episode" => Some(Placeholder::Episode),
            "episode_title" => Some(Placeholder::EpisodeTitle),
            _ => None,
        }
    }

    /// The name of the placeholder, as it is written in a format.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::Placeholder;
    ///
    /// assert_eq!(Placeholder::Extension.as_str(), "ext");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            Placeholder::Name => "name",
            Placeholder::Extension => "ext",
            Placeholder::Parent => "parent",
            Placeholder::Counter => "n",
            Placeholder::Date => "date",
            Placeholder::Time => "time",
            Placeholder::Size => "size",
            Placeholder::Hash => "hash",
            Placeholder::Random => "rand",
            Placeholder::Upper => "upper",
            Placeholder::Lower => "lower",
            Placeholder::Capitalised => "case",
            Placeholder::Title => "title",
            Placeholder::Series => "series",
            Placeholder::Season => "season",
            Placeholder::Episode => "episode",
            Placeholder::EpisodeTitle => "episode_title",
        }
    }
}

/// The values that the placeholders are replaced with.
pub(crate) struct Values<'a> {
    /// The file the name is formed for.
    pub input: &'a RenameInput,

    /// The time of the rename.
    pub now: SystemTime,

    /// The number of the counter, if the rule carries one.
    pub counter: Option<i64>,

    /// The values of the series, if the rule carries them.
    pub series: Option<&'a SeriesValues>,

    /// The width of the season number.
    pub season_width: usize,

    /// The width of the episode number.
    pub episode_width: usize,
}

/// Replaces the placeholders of a format by their values.
///
/// WHY: a placeholder that is not known stays in the name as it is. A file
///       whose name carries a square bracket of its own — "notes[1].txt" —
///       therefore keeps its name.
pub(crate) fn evaluate(format: &str, values: &Values<'_>) -> Result<String, TemplateError> {
    let mut out = String::with_capacity(format.len());
    let mut rest = format;

    while let Some(start) = rest.find('[') {
        out.push_str(&rest[..start]);
        let tail = &rest[start + 1..];
        let end = match tail.find(']') {
            Some(end) => end,
            None => {
                // WHY: a bracket without a partner is a bracket of the name
                // itself and not the start of a placeholder.
                out.push('[');
                rest = tail;
                continue;
            }
        };

        let name = &tail[..end];
        match Placeholder::parse(name) {
            Some(placeholder) => out.push_str(&value_of(placeholder, values)?),
            None => {
                out.push('[');
                out.push_str(name);
                out.push(']');
            }
        }
        rest = &tail[end + 1..];
    }

    out.push_str(rest);
    Ok(out)
}

/// The value of one placeholder.
fn value_of(placeholder: Placeholder, values: &Values<'_>) -> Result<String, TemplateError> {
    match placeholder {
        Placeholder::Name => Ok(base_name_of(values.input)),
        Placeholder::Extension => Ok(extension_of(values.input)),
        Placeholder::Parent => Ok(parent_name_of(values.input)),
        Placeholder::Counter => Ok(COUNTER_MARKER.to_string()),
        Placeholder::Date => Ok(crate::datetime::format_date(values.now)),
        Placeholder::Time => Ok(crate::datetime::format_time(values.now)),
        Placeholder::Size => match values.input.size {
            Some(size) => Ok(size.to_string()),
            None => Err(unknown(values, "size")),
        },
        Placeholder::Hash => match values.input.checksum {
            Some(checksum) => Ok(short_checksum(checksum)),
            None => Err(unknown(values, "hash")),
        },
        Placeholder::Random => Ok(match &values.input.random {
            Some(value) => value.clone(),
            None => derived_random(values.input),
        }),
        Placeholder::Upper => Ok(base_name_of(values.input).to_uppercase()),
        Placeholder::Lower => Ok(base_name_of(values.input).to_lowercase()),
        Placeholder::Capitalised => Ok(capitalise(&base_name_of(values.input))),
        Placeholder::Title => Ok(title_case(&base_name_of(values.input))),
        Placeholder::Series => match values.series {
            Some(series) => Ok(series.name.clone()),
            None => Err(unknown(values, "series")),
        },
        Placeholder::Season => match values.series.and_then(|series| series.season) {
            Some(season) => Ok(pad_number(season, values.season_width)),
            None => Err(unknown(values, "season")),
        },
        Placeholder::Episode => match values.series.and_then(|series| series.episode) {
            Some(episode) => Ok(pad_number(episode, values.episode_width)),
            None => Err(unknown(values, "episode")),
        },
        Placeholder::EpisodeTitle => match values.series {
            Some(series) => Ok(series.title.clone()),
            None => Err(unknown(values, "episode_title")),
        },
    }
}

/// The failure that reports a value that the caller did not supply.
fn unknown(values: &Values<'_>, placeholder: &'static str) -> TemplateError {
    TemplateError::UnknownValue {
        placeholder,
        path: values.input.path.clone(),
    }
}

/// Writes a number with the given number of digits.
pub(crate) fn pad_number(value: u64, width: usize) -> String {
    format!("{value:0width$}", width = width)
}

/// The first eight characters of the checksum of a file.
///
/// WHY: the checksum of a file is a number that is written in hexadecimal. A
///       name carries only the first eight characters of it, because a name
///       with sixteen hexadecimal characters is longer than the name of the
///       file usually is.
pub(crate) fn short_checksum(checksum: u64) -> String {
    let text = format!("{checksum:x}");
    text.chars().take(8).collect()
}

/// The value of the placeholder `[rand]`, derived from the name of the file.
///
/// WHY: a real random number would break the guarantee of FR-33: the same
///       file would receive a different name every time the rule runs. The
///       value is therefore a hash of the name of the file, and the hash is
///       the FNV-1a — a small, documented algorithm whose result never
///       changes. The default hasher of the standard library is not used,
///       because its result is not guaranteed to be the same in every version
///       of the compiler.
pub(crate) fn derived_random(input: &RenameInput) -> String {
    let hash = fnv1a(file_name_of(input).as_bytes());
    // WHY: the value has exactly eight characters. The width of a format is
    //       a *minimum*, so a hash of sixty-four bits would be written with
    //       sixteen characters and the name would grow with it. The lower
    //       thirty-two bits carry the value, and the upper ones are dropped.
    format!("{:08x}", hash & 0xffff_ffff)
}

/// The FNV-1a hash of the bytes.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// The text with its first letter in upper case.
pub(crate) fn capitalise(text: &str) -> String {
    let mut characters = text.chars();
    match characters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
    }
}

/// The text with every word in title case.
pub(crate) fn title_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at_word_start = true;
    for character in text.chars() {
        if character.is_whitespace() {
            at_word_start = true;
            out.push(character);
            continue;
        }
        if at_word_start {
            out.extend(character.to_uppercase());
            at_word_start = false;
        } else {
            out.extend(character.to_lowercase());
        }
    }
    out
}
