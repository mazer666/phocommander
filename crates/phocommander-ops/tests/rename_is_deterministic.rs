//! The determinism of the rename rule (FR-33).
//!
//! WHAT: the test runs the same rule twice over the same files and compares
//!       the results.
//! WHY: the criterion of the milestone asks for the same selection and the
//!       same rule to produce the same names in two consecutive runs, and for
//!       the proof that the result is a pure function of the input: no global
//!       state, no hidden time and no random source that is not declared as
//!       such.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test rename_is_deterministic -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, SystemTime};

use phocommander_ops::template::{
    CounterRule, RenameInput, RenameRule, ReplaceRule, SeriesRule, form_names,
};

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// A rule that uses every part of the engine.
fn rule() -> RenameRule {
    RenameRule::new("[rand] - [name] - [n] - [parent] - [date] - [size]")
        .with_counter(CounterRule::new(1, 1, 3))
        .with_replacement(ReplaceRule::literal(" ", "_"))
        .with_series(SeriesRule::new().with_season(1))
}

/// The files of the selection.
fn inputs() -> Vec<RenameInput> {
    vec![
        RenameInput::new("/tv/The Expanse/Duldung.avi").with_size(4096),
        RenameInput::new("/tv/The Expanse/Leviathan.avi").with_size(8192),
        RenameInput::new("/tv/The Expanse/Homegrown.avi").with_size(16),
    ]
}

#[test]
fn rename_is_deterministic() {
    // arrange
    let rule = rule();
    let inputs = inputs();

    // act
    let first = form_names(&rule, &inputs, moment());
    let second = form_names(&rule, &inputs, moment());

    // assert
    let first = match first {
        Ok(names) => names,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    let second = match second {
        Ok(names) => names,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert_eq!(first, second);
}

#[test]
fn the_result_does_not_depend_on_a_hidden_clock() {
    // arrange
    // WHY: the rule carries no date and no time, so a different moment may
    //       not change the names.
    let rule = RenameRule::new("[name] - [n]").with_counter(CounterRule::new(1, 1, 3));
    let inputs = inputs();

    // act
    let first = form_names(&rule, &inputs, moment());
    let later = form_names(
        &rule,
        &inputs,
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_900_000_000),
    );

    // assert
    assert_eq!(first, later);
}

#[test]
fn the_result_does_not_depend_on_a_hidden_random_source() {
    // arrange
    let rule = RenameRule::new("[rand]");
    let inputs = inputs();

    // act
    let first = form_names(&rule, &inputs, moment());
    let second = form_names(&rule, &inputs, moment());

    // assert
    assert_eq!(first, second);
}

#[test]
fn the_result_does_not_depend_on_the_order_of_the_calls() {
    // arrange
    let rule = rule();
    let inputs = inputs();

    // act
    let alone = form_names(&rule, &inputs, moment());
    let _ = form_names(&RenameRule::new("[name]"), &inputs, SystemTime::UNIX_EPOCH);
    let again = form_names(&rule, &inputs, moment());

    // assert
    // WHY: a rule that runs between two runs of the same rule may not change
    //       the result — there is no global state that a rule could leave
    //       behind.
    assert_eq!(alone, again);
}
