//! The detection of a season and an episode number in an existing name.
//!
//! WHAT: the numbers of a series episode ([`SeriesNumbers`]), the function
//!       that recognises them in a name ([`detect_series_numbers`]) and the
//!       function that reads the series name from the parent directory
//!       ([`series_name_from_parent`]).
//! WHY: the template of a series names its files
//!       `{serie} - S{staffel:02}E{episode:02} - {titel}.{format}`. When the
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
/// assert_eq!(numbers.map(|numbers| numbers.season), Some(1));
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

/// Recognises the pattern `SxxEyy` in a name.
///
/// WHY: the pattern is the one the template of the specification uses. The
/// search ignores the case, because a file that was named by another
/// program may carry a lower case `s`.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::series::detect_series_numbers;
///
/// assert!(detect_series_numbers("Show.S02E10.mkv").is_some());
/// assert!(detect_series_numbers("Show.mkv").is_none());
/// ```
pub fn detect_series_numbers(name: &str) -> Option<SeriesNumbers> {
    let characters: Vec<char> = name.chars().collect();
    let mut index = 0;

    while index + 3 < characters.len() {
        if characters[index].eq_ignore_ascii_case(&'S')
            && characters[index + 1].is_ascii_digit()
            && characters[index + 2].eq_ignore_ascii_case(&'E')
            && characters[index + 3].is_ascii_digit()
        {
            let season = read_number(&characters, index + 1);
            let episode = read_number(&characters, index + 3);
            return Some(SeriesNumbers { season, episode });
        }
        index += 1;
    }

    None
}

/// Reads the number that starts at `start`.
fn read_number(characters: &[char], start: usize) -> u64 {
    let mut number = 0u64;
    let mut index = start;
    while index < characters.len() && characters[index].is_ascii_digit() {
        number = number.saturating_mul(10).saturating_add(
            u64::from(characters[index].to_digit(10).unwrap_or(0)),
        );
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
    fn a_name_without_a_parent_has_no_series_name() {
        // act, assert
        assert!(series_name_from_parent(Path::new("S01E01.mkv")).is_none());
    }
}
