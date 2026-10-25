//! The detection of the season and the episode in an existing name (FR-37).
//!
//! WHAT: the test names three files — "The.Expanse.S01E03.avi",
//!       "show.s02e11.mkv" and "Duldung.avi" — and reads the season and the
//!       episode out of them.
//! WHY: the criterion of the milestone asks for the season 1 and the episode
//!       3 in the first name, the season 2 and the episode 11 in the second
//!       and no season and no episode in the third. The dialog offers the
//!       numbers it found as the pre-filled values, and the engine uses them
//!       when it forms the name.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test series_detects_season_and_episode -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_ops::series::detect_series_numbers;
use std::time::{Duration, SystemTime};

use phocommander_ops::template::{RenameInput, RenameRule, SeriesRule, form_names};

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

#[test]
fn series_detects_season_and_episode() {
    // arrange
    let first = "The.Expanse.S01E03.avi";
    let second = "show.s02e11.mkv";
    let third = "Duldung.avi";

    // act, assert
    // WHY: the pattern is recognised regardless of the case, because a file
    //       that was named by another program may carry a lower case `s`.
    let numbers = detect_series_numbers(first);
    assert_eq!(numbers.map(|numbers| numbers.season), Some(1));
    assert_eq!(numbers.map(|numbers| numbers.episode), Some(3));

    let numbers = detect_series_numbers(second);
    assert_eq!(numbers.map(|numbers| numbers.season), Some(2));
    assert_eq!(numbers.map(|numbers| numbers.episode), Some(11));

    let numbers = detect_series_numbers(third);
    assert_eq!(numbers.map(|numbers| numbers.season), None);
    assert_eq!(numbers.map(|numbers| numbers.episode), None);
}

#[test]
fn the_engine_uses_the_numbers_that_it_found_in_the_name() {
    // arrange
    let rule = RenameRule::new("").with_series(SeriesRule::new());
    let inputs = vec![
        RenameInput::new("/tv/The Expanse/The.Expanse.S01E03.avi"),
        RenameInput::new("/tv/The Expanse/show.s02e11.mkv"),
    ];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let names = match names {
        Ok(names) => names,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert_eq!(
        names[0].name(),
        Some("The Expanse - S01E03 - The.Expanse.avi")
    );
    assert_eq!(names[1].name(), Some("The Expanse - S02E11 - show.mkv"));
}

#[test]
fn a_file_without_a_pattern_carries_no_numbers() {
    // arrange
    let rule = RenameRule::new("").with_series(SeriesRule::new());
    let inputs = vec![RenameInput::new("/tv/The Expanse/Duldung.avi")];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let names = match names {
        Ok(names) => names,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert!(names[0].name().is_none());
    assert!(matches!(
        names[0].problem(),
        Some(phocommander_ops::template::TemplateError::UnknownValue {
            placeholder: "season",
            ..
        })
    ));
}
