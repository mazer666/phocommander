//! The name of the series comes from the parent directory (FR-38).
//!
//! WHAT: the test names the file "episode.avi" in the directory
//!       "The Expanse" and reads the name of the series out of it.
//! WHY: the criterion of the milestone asks for the pre-filled name of the
//!       series to be "The Expanse" — the name of the directory the files
//!       live in — and for an entry that overrides it with another name. The
//!       override is the value the rename uses.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test series_name_defaults_to_the_parent -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_ops::series::series_name_from_parent;
use std::path::Path;

use phocommander_ops::template::{RenameInput, RenameRule, SeriesRule, form_names};
use std::time::{Duration, SystemTime};

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// The name that the rule forms for the file.
fn formed(rule: &RenameRule, input: &RenameInput) -> String {
    let names = form_names(rule, std::slice::from_ref(input), moment());
    let names = match names {
        Ok(names) => names,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    match names.first().and_then(|formed| formed.name()) {
        Some(name) => name.to_string(),
        None => panic!("the name should have been formed"),
    }
}

#[test]
fn series_name_defaults_to_the_parent() {
    // arrange
    let path = Path::new("/tv/The Expanse/episode.avi");

    // act, assert
    // WHY: the name of the series is the name of the directory the files
    //       live in.
    let name = series_name_from_parent(path);
    assert_eq!(name, Some("The Expanse".to_string()));

    let rule = RenameRule::new("").with_series(SeriesRule::new().with_season(1).with_episode(1));
    let input = RenameInput::new(path);
    assert_eq!(formed(&rule, &input), "The Expanse - S01E01 - episode.avi");
}

#[test]
fn the_name_of_the_series_can_be_overridden() {
    // arrange
    let rule = RenameRule::new("").with_series(
        SeriesRule::new()
            .with_series_name("Duldung")
            .with_season(1)
            .with_episode(1),
    );
    let input = RenameInput::new("/tv/Collection/episode.avi");

    // act, assert
    // WHY: a series that is filed in a collection folder has a directory that
    //       is not its name, and the user may change the value.
    assert_eq!(formed(&rule, &input), "Duldung - S01E01 - episode.avi");
}
