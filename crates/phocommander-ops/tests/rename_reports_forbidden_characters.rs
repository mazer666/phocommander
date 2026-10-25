//! The characters that the target platform forbids (FR-40).
//!
//! WHAT: the test forms the name "A:B<C" and reads the characters that the
//!       platform forbids out of it, together with the replacement that is
//!       suggested.
//! WHY: the criterion of the milestone asks for every character that the
//!       target platform forbids to be reported and for a replacement to be
//!       proposed. Nothing may be changed silently — the user decides. The
//!       part of the criterion that belongs to the dialog — that the rename
//!       waits for the confirmation of the user — is proved by the dialog.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test rename_reports_forbidden_characters -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_ops::names::{invalid_characters, is_reserved_device_name, suggest_replacement};
use std::time::{Duration, SystemTime};

use phocommander_ops::template::{RenameInput, RenameRule, form_names, name_problems};

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

#[test]
fn rename_reports_forbidden_characters() {
    // arrange
    let rule = RenameRule::new("[name]<[ext]");
    let input = RenameInput::new("/photos/A:B.avi");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let names = match names {
        Ok(names) => names,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    let name = match names[0].name() {
        Some(name) => name.to_string(),
        None => panic!("the name should have been formed"),
    };
    assert_eq!(name, "A:B<avi");

    // WHY: every character that the platform forbids is reported, and every
    //       character comes with the replacement that is suggested.
    let problems = name_problems(name.as_str());
    assert_eq!(problems.len(), 2);
    assert_eq!(problems[0], (':', '-'));
    assert_eq!(problems[1], ('<', '('));
}

#[test]
fn the_replacement_of_a_forbidden_character_is_allowed() {
    // act, assert
    for character in phocommander_ops::names::WINDOWS_FORBIDDEN_CHARACTERS {
        let replacement = suggest_replacement(character);
        assert!(
            !phocommander_ops::names::WINDOWS_FORBIDDEN_CHARACTERS.contains(&replacement),
            "{character}"
        );
    }
}

#[test]
fn a_name_without_a_forbidden_character_reports_nothing() {
    // act, assert
    assert!(name_problems("The Expanse - S01E01 - Duldung.avi").is_empty());
    assert!(invalid_characters("The Expanse - S01E01 - Duldung.avi").is_empty());
}

#[test]
fn a_device_name_of_windows_is_reported() {
    // act, assert
    assert!(is_reserved_device_name("NUL.txt"));
    assert!(!is_reserved_device_name("Duldung.avi"));
}
