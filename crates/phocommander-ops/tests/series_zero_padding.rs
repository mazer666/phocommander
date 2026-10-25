//! The zero padding of the season and the episode number (FR-39).
//!
//! WHAT: the test names the files of a series episode and reads the width of
//!       the numbers out of the names that the engine forms.
//! WHY: the criterion of the milestone asks for the name "S01E01" for the
//!       season 1 and the episode 1, and for the width to grow to three as
//!       soon as the selection carries a number with three digits — so that
//!       every file of one run carries the same width.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test series_zero_padding -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, SystemTime};

use phocommander_ops::template::{CounterRule, RenameInput, RenameRule, SeriesRule, form_names};

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// The names that the rule forms for the files.
fn formed(rule: &RenameRule, inputs: &[RenameInput]) -> Vec<String> {
    let names = form_names(rule, inputs, moment());
    let names = match names {
        Ok(names) => names,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    names
        .iter()
        .map(|formed| match formed.name() {
            Some(name) => name.to_string(),
            None => panic!("the name should have been formed"),
        })
        .collect()
}

#[test]
fn series_zero_padding() {
    // arrange
    let rule = RenameRule::new("").with_series(SeriesRule::new().with_season(1).with_episode(1));
    let inputs = vec![RenameInput::new("/tv/The Expanse/Duldung.avi")];

    // act
    let names = formed(&rule, &inputs);

    // assert
    // WHY: the season and the episode are written with two digits, so that
    //       the result is "S01E01" and not "S1E1".
    assert_eq!(names, vec!["The Expanse - S01E01 - Duldung.avi"]);
}

#[test]
fn the_width_grows_with_the_largest_number_of_the_selection() {
    // arrange
    let rule = RenameRule::new("")
        .with_series(SeriesRule::new().with_season(1))
        .with_counter(CounterRule::new(99, 1, 3));
    let inputs = vec![
        RenameInput::new("/tv/The Expanse/a.avi"),
        RenameInput::new("/tv/The Expanse/b.avi"),
    ];

    // act
    let names = formed(&rule, &inputs);

    // assert
    // WHY: the selection carries the episode 100, and every file of the run
    //       is therefore named with three digits.
    assert_eq!(
        names,
        vec![
            "The Expanse - S01E099 - a.avi",
            "The Expanse - S01E100 - b.avi",
        ]
    );
}

#[test]
fn the_width_of_the_season_grows_as_well() {
    // arrange
    let rule = RenameRule::new("").with_series(SeriesRule::new().with_episode(1));
    let inputs = vec![RenameInput::new("/tv/The Expanse/The.Expanse.S123E01.avi")];

    // act
    let names = formed(&rule, &inputs);

    // assert
    assert_eq!(names, vec!["The Expanse - S123E01 - The.Expanse.avi"]);
}
