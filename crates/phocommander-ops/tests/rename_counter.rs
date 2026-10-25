//! The counter of the rename rule (FR-32).
//!
//! WHAT: the test numbers the files of a selection and reads the numbers out
//!       of the names that the engine forms.
//! WHY: the criterion of the milestone asks for the names "001", "002" and
//!       "003" for the start value 1, the step 1 and the width 3.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test rename_counter -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, SystemTime};

use phocommander_ops::template::{CounterOrder, CounterRule, RenameInput, RenameRule, form_names};

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
fn rename_counter() {
    // arrange
    let rule = RenameRule::new("[n]").with_counter(CounterRule::new(1, 1, 3));
    let inputs = vec![
        RenameInput::new("/photos/a.avi"),
        RenameInput::new("/photos/b.avi"),
        RenameInput::new("/photos/c.avi"),
    ];

    // act
    let names = formed(&rule, &inputs);

    // assert
    assert_eq!(names, vec!["001", "002", "003"]);
}

#[test]
fn the_counter_starts_at_the_configured_value_and_grows_by_the_step() {
    // arrange
    let rule = RenameRule::new("[n]").with_counter(CounterRule::new(10, 5, 2));
    let inputs = vec![
        RenameInput::new("/photos/a.avi"),
        RenameInput::new("/photos/b.avi"),
        RenameInput::new("/photos/c.avi"),
    ];

    // act
    let names = formed(&rule, &inputs);

    // assert
    assert_eq!(names, vec!["10", "15", "20"]);
}

#[test]
fn the_counter_can_count_by_the_name() {
    // arrange
    let rule = RenameRule::new("[n] - [name]")
        .with_counter(CounterRule::new(1, 1, 1).with_order(CounterOrder::Name));
    let inputs = vec![
        RenameInput::new("/photos/c.avi"),
        RenameInput::new("/photos/a.avi"),
        RenameInput::new("/photos/b.avi"),
    ];

    // act
    let names = formed(&rule, &inputs);

    // assert
    assert_eq!(names, vec!["3 - c", "1 - a", "2 - b"]);
}
