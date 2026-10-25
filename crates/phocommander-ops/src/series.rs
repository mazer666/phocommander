//! The detection of a season and an episode number in an existing name.
//!
//! WHAT: the numbers of a series episode ([{SeriesNumbers}]), the pattern of a
//!       series episode with the place it was found ([{SeriesPattern}]), the
//!       function that recognises them in a name ([{detect_series_numbers}])
//!       and the function that reads the series name from the parent
//!       directory ([{series_name_from_parent}]).
//! WHY: the template of a series names its files
//!       {{serie} - S{staffel:02}E{episode:02} - {titel}.{format}}. When the
//!       user renames a file that already carries such a number, the dialog
//!       offers the numbers it found as the pre-filled values (FR-37), and
//!       the name of the series is the name of the directory the files live
//!       in (FR-38). Both are read here, so that the dialog and the
//!       operations agree about them.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::series::detect_series_numbers;
//!
//! let numbers = detect_series_numbers("The.Expanse.S01E03.1080p.mkv");
//! assert_eq!(numbers.map(|numbers| numbers.season), Some(1));
//! assert_eq!(numbers.map(|numbers| numbers.episode), Some(3));
//! ```

use std::path::Path;

/// The season and the episode number of a series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeriesNumbers {
    /// The number of the season.
    pub season: u64,

    /// The number of the episode.
    pub episode: u64,
}

/// The season and the episode number of a series, with the place they were
/// found in the name.
///
/// WHY: the place matters for the title of an episode: the name of an
///       episode that is already filed carries the pattern between the parts
///       of the name — "Show.S01E03.Title" — and the title is what is left
///       of it when the pattern is cut out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeriesPattern {
    /// The number of the season.
    pub season: u64,

    /// The number of the episode.
    pub episode: u64,

    /// The first byte of the pattern in the name.
    pub start: usize,

    /// The byte behind the pattern in the name.
    pub end: usize,
}

/// Recognises the pattern {SxxEyy} in a name.
///
/// WHY: the pattern is the one the template of the specification uses. The
/// search ignores the case, because a file that was named by another
/// program may carry a lower case {s}.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::series::detect_series_numbers;
///
/// assert!(detect_series_numbers("Show.S02E10.mkv").is_some());
/// assert!(detect_series_numbers("Show.mkv").is_none());
/// ```
pub fn detect_series_numbers(name: &str) -> Option<SeriesNumbers> {
    detect_series_pattern(name).map(|pattern| SeriesNumbers {
        season: pattern.season,
        episode: pattern.episode,
    })
}

/// Recognises the pattern {SxxEyy} in a name and reports where it is.
///
/// WHY: the place of the pattern is needed to cut the title of an episode out
///       of a name that already carries the pattern. The numbers themselves
///       are read by [{detect_series_numbers}], which is the same search
///       without the place.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::series::detect_series_pattern;
///
/// let pattern = detect_series_pattern("Show.S01E03.Title");
/// assert_eq!(pattern.map(|pattern| pattern.start), Some(5));
/// assert_eq!(pattern.map(|pattern| pattern.end), Some(11));
/// ```
pub fn detect_series_pattern(name: &str) -> Option<SeriesPattern> {
    let characters: Vec<char> = name.chars().collect();
    let mut index = 0;

    while index < characters.len() {
        if !characters[index].eq_ignore_ascii_case(&'S') {
            index += 1;
            continue;
        }

        // WHY: the season may have more than one digit, so the number is
        // read first and the `E` is looked for behind it. The pattern of the
        // specification is `SxxEyy`, and a season of two or three digits is
        // as common as a two digit episode.
        let season_start = index + 1;
        let season_end = end_of_number(&characters, season_start);
        if season_end == season_start {
            // No digit follows the `S`, so this is not a season marker.
            index += 1;
            continue;
        }

        if season_end < characters.len() && characters[season_end].eq_ignore_ascii_case(&'E') {
            let episode_start = season_end + 1;
            let episode_end = end_of_number(&characters, episode_start);
            if episode_end > episode_start {
                return Some(SeriesPattern {
                    season: read_number(&characters, season_start),
                    episode: read_number(&characters, episode_start),
                    start: byte_offset(&characters, index),
                    end: byte_offset(&characters, episode_end),
                });
            }
        }

        index += 1;
    }

    None
}

/// The byte offset of the character at the given position.
///
/// WHY: the place of the pattern is given in bytes, because a name is cut
///       with byte offsets. The characters before the place may be more than
///       one byte long, and their length has to be added up.
fn byte_offset(characters: &[char], index: usize) -> usize {
    characters[..index]
        .iter()
        .map(|character| character.len_utf8())
        .sum()
}

/// Returns the position behind the number that starts at {start}.
///
/// WHY: the recognition has to know where the season number ends, because
/// the `E` of the pattern follows it. The number may have two or three
/// digits, so its end cannot be assumed.
fn end_of_number(characters: &[char], start: usize) -> usize {
    let mut index = start;
    while index < characters.len() && characters[index].is_ascii_digit() {
        index += 1;
    }
    index
}

/// Reads the number that starts at {start}.
fn read_number(characters: &[char], start: usize) -> u64 {
    let mut number = 0u64;
    let mut index = start;
    while index < characters.len() && characters[index].is_ascii_digit() {
        number = number
            .saturating_mul(10)
            .saturating_add(u64::from(characters[index].to_digit(10).unwrap_or(0)));
        index += 1;
    }
    number
}

/// The name of the series, taken from the parent directory.
///
/// WHY: the series name is the name of the directory the files live in, and
/// it is always shown before the rename so that the user can change it
/// (FR-38).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::series::series_name_from_parent;
/// use std::path::Path;
///
/// assert_eq!(
///     series_name_from_parent(Path::new("/tv/The Expanse/S01E01.mkv")),
///     Some("The Expanse".to_string())
/// );
/// ```
pub fn series_name_from_parent(path: &Path) -> Option<String> {
    let parent = path.parent()?;
    let name = parent.file_name()?.to_string_lossy().to_string();
    if name.is_empty() {
        return None;
    }
    Some(name)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_pattern_is_recognised_in_a_name() {
        // act
        let numbers = detect_series_numbers("The.Expanse.S01E03.1080p.mkv");

        // assert
        let numbers = match numbers {
            Some(numbers) => numbers,
            None => panic!("the pattern should have been recognised"),
        };
        assert_eq!(numbers.season, 1);
        assert_eq!(numbers.episode, 3);
    }

    #[test]
    fn a_two_digit_season_is_recognised() {
        // WHY: the season may have two digits, so the `E` of the pattern
        // follows the number and not the `S`. This case was found by the
        // test above.
        // act
        let numbers = detect_series_numbers("Show.S12E03.mkv");

        // assert
        let numbers = match numbers {
            Some(numbers) => numbers,
            None => panic!("the pattern should have been recognised"),
        };
        assert_eq!(numbers.season, 12);
        assert_eq!(numbers.episode, 3);
    }

    #[test]
    fn the_pattern_is_recognised_with_two_and_three_digits() {
        // act
        let numbers = detect_series_numbers("Show.S12E103.mkv");

        // assert
        let numbers = match numbers {
            Some(numbers) => numbers,
            None => panic!("the pattern should have been recognised"),
        };
        assert_eq!(numbers.season, 12);
        assert_eq!(numbers.episode, 103);
    }

    #[test]
    fn the_pattern_is_recognised_without_the_case() {
        // act
        let numbers = detect_series_numbers("show.s02e10.mkv");

        // assert
        assert_eq!(
            numbers,
            Some(SeriesNumbers {
                season: 2,
                episode: 10
            })
        );
    }

    #[test]
    fn a_name_without_the_pattern_is_not_recognised() {
        // act, assert
        assert!(detect_series_numbers("Show.mkv").is_none());
        assert!(detect_series_numbers("S01.mkv").is_none());
        assert!(detect_series_numbers("Season 1 Episode 3.mkv").is_none());
    }

    #[test]
    fn the_series_name_is_the_name_of_the_parent_directory() {
        // act
        let name = series_name_from_parent(Path::new("/tv/The Expanse/S01E01.mkv"));

        // assert
        assert_eq!(name, Some("The Expanse".to_string()));
    }

    #[test]
    fn the_pattern_reports_the_place_that_it_was_found() {
        // act
        let pattern = detect_series_pattern("Show.S01E03.Title");

        // assert
        let pattern = match pattern {
            Some(pattern) => pattern,
            None => panic!("the pattern should have been recognised"),
        };
        assert_eq!(pattern.season, 1);
        assert_eq!(pattern.episode, 3);
        assert_eq!(pattern.start, 5);
        assert_eq!(pattern.end, 11);
        assert_eq!(&"Show.S01E03.Title"[pattern.start..pattern.end], "S01E03");
    }

    #[test]
    fn a_name_without_a_pattern_has_no_place() {
        // act, assert
        assert!(detect_series_pattern("Duldung.avi").is_none());
    }

    #[test]
    fn a_name_without_a_parent_has_no_series_name() {
        // act, assert
        assert!(series_name_from_parent(Path::new("S01E01.mkv")).is_none());
    }
}
