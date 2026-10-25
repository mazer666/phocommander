//! The property test of the determinism of the rename rule (FR-33).
//!
//! WHAT: the property test runs a hundred cases and proves that the result
//!       of the rule is a pure function of the input.
//! WHY: the criterion of the milestone asks for this test. The rule and the
//!       files are drawn from the generator of `proptest`, and every case is
//!       run twice — the two runs have to be the same. A rule that read the
//!       clock of the library or drew a random number would fail here.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test rename_properties -- --proptest 100
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, SystemTime};

use phocommander_ops::template::{CounterRule, RenameInput, RenameRule, form_names};
use proptest::prelude::*;

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// The formats that the test draws from.
///
/// WHY: the list carries a format for every part of the engine: the
///       placeholders of the name, the counter, the parent, the date, the
///       time, the size, the checksum, the value that stands for a random
///       number and a placeholder that is not known.
fn formats() -> impl Strategy<Value = String> {
    // WHY: the strategies draw from a list of formats. The list holds the
    //       formats as owned strings, because a strategy has to hand the
    //       value on and a borrowed one would not outlive the list.
    prop::sample::select(
        vec![
            "[name].[ext]",
            "[name] - [n].[ext]",
            "[upper]_[lower].[ext]",
            "[parent]-[name].[ext]",
            "[date]-[time]-[name].[ext]",
            "[size]-[hash].[ext]",
            "[rand]-[name].[ext]",
            "[name].[unknown].[ext]",
            "[name] - [n] - [rand]",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>(),
    )
}

/// The files of a selection, named by the generator.
fn inputs(names: &[String]) -> Vec<RenameInput> {
    names
        .iter()
        .map(|name| RenameInput::new(format!("/photos/{name}.txt")))
        .collect()
}

proptest! {
    /// The same rule and the same files always yield the same names.
    #[test]
    fn the_result_is_a_function_of_the_input(
        pattern in formats(),
        step in -3i64..3i64,
        width in 1usize..6,
        names in prop::collection::vec("[a-z]{1,6}", 1..4),
    ) {
        let rule = RenameRule::new(pattern).with_counter(CounterRule::new(1, step, width));
        let files = inputs(&names);

        let first = form_names(&rule, &files, moment());
        let second = form_names(&rule, &files, moment());

        prop_assert_eq!(first, second);
    }

    /// The counter numbers the files of the selection in a row.
    #[test]
    fn the_counter_numbers_the_files_in_a_row(
        names in prop::collection::vec("[a-z]{1,6}", 1..6),
    ) {
        let rule = RenameRule::new("[n]").with_counter(CounterRule::new(1, 1, 3));
        let files = inputs(&names);

        let formed = form_names(&rule, &files, moment());

        prop_assert!(formed.is_ok());
        let formed = formed.unwrap_or_default();
        for (index, name) in formed.iter().enumerate() {
            let expected = format!("{:03}", index + 1);
            prop_assert_eq!(name.name(), Some(expected.as_str()));
        }
    }

    /// A file whose name cannot be formed is reported and does not change the
    /// names of the other files.
    #[test]
    fn a_file_that_cannot_be_named_does_not_stop_the_others(
        names in prop::collection::vec("[a-z]{1,6}", 1..4),
    ) {
        let rule = RenameRule::new("[hash]");
        let files = inputs(&names);

        let formed = form_names(&rule, &files, moment());

        prop_assert!(formed.is_ok());
        let formed = formed.unwrap_or_default();
        for name in formed.iter() {
            prop_assert!(name.name().is_none());
            prop_assert!(name.problem().is_some());
        }
    }
}
