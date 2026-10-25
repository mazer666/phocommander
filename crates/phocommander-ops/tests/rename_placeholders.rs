//! The placeholders of the rename rule (FR-30).
//!
//! WHAT: the test names one file and reads every placeholder out of the name
//!       that the engine forms from it.
//! WHY: the criterion of the milestone asks for one case per placeholder:
//!       the name, the extension, the parent, the counter, the date, the
//!       time, the size, the short checksum, the random value and the case
//!       conversion. Every case fixes the file it works on and the moment of
//!       the rename, so that the expected value is a fact and not a guess.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test rename_placeholders -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, SystemTime};

use phocommander_ops::template::{CounterRule, RenameInput, RenameRule, form_names};

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// The name that the format forms for the file.
fn formed(format: &str, input: &RenameInput) -> String {
    let names = form_names(
        &RenameRule::new(format),
        std::slice::from_ref(input),
        moment(),
    );
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
fn rename_placeholders() {
    // arrange
    let input = RenameInput::new("/photos/Duldung.avi")
        .with_size(4096)
        .with_checksum(0x1234_5678_9abc_def0);

    // act, assert
    assert_eq!(formed("[name]", &input), "Duldung");
    assert_eq!(formed("[ext]", &input), "avi");
    assert_eq!(formed("[parent]", &input), "photos");
    assert_eq!(formed("[date]", &input), "2023-11-14");
    assert_eq!(formed("[time]", &input), "22-13-20");
    assert_eq!(formed("[size]", &input), "4096");
    assert_eq!(formed("[hash]", &input), "12345678");
    assert_eq!(formed("[upper]", &input), "DULDUNG");
    assert_eq!(formed("[lower]", &input), "duldung");
    assert_eq!(formed("[case]", &input), "Duldung");
    assert_eq!(formed("[title]", &input), "Duldung");
}

#[test]
fn the_counter_placeholder_carries_the_number_of_the_counter() {
    // arrange
    let rule = RenameRule::new("[n]").with_counter(CounterRule::new(1, 1, 3));
    let inputs = vec![
        RenameInput::new("/photos/a.avi"),
        RenameInput::new("/photos/b.avi"),
    ];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let names = match names {
        Ok(names) => names,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert_eq!(names[0].name(), Some("001"));
    assert_eq!(names[1].name(), Some("002"));
}

#[test]
fn the_random_value_is_derived_from_the_name_and_does_not_change() {
    // arrange
    let input = RenameInput::new("/photos/Duldung.avi");

    // act
    let first = formed("[rand]", &input);
    let second = formed("[rand]", &input);

    // assert
    // WHY: a real random number would break the determinism of FR-33. The
    //       value is derived from the name of the file, and it is the same in
    //       every run.
    assert_eq!(first, second);
    assert_eq!(first.len(), 8);
    assert!(
        first.chars().all(|character| character.is_ascii_hexdigit()),
        "{first}"
    );
}

#[test]
fn the_random_value_of_two_files_differs() {
    // arrange
    let first = RenameInput::new("/photos/a.avi");
    let second = RenameInput::new("/photos/b.avi");

    // act, assert
    assert_ne!(formed("[rand]", &first), formed("[rand]", &second));
}
