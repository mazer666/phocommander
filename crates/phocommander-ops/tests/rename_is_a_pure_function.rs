//! The determinism of the rename rule, proved as a property (FR-33).
//!
//! WHAT: the test draws many rules and many selections at random and proves
//!       that the formation of the names is a pure function of its input.
//! WHY: the criterion of the specification asks for a property test. The
//!       four example tests of the file `rename_is_deterministic.rs` prove
//!       the property for the cases that its author thought of. A property
//!       test proves it for the cases that nobody thought of — the rule with
//!       a strange combination of the placeholders, the selection with a
//!       name that carries a character that no other name carries.
//!
//!       The promise of FR-33 is that the same input always yields the same
//!       output. That promise has to hold for every input, and a test with
//!       fixed cases can only show that it holds for those cases.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test rename_is_a_pure_function
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, SystemTime};

use phocommander_ops::template::{
    CounterRule, FormedName, RenameInput, RenameRule, ReplaceRule, form_names,
};
use proptest::prelude::*;

/// The moment that the test fixes.
///
/// WHY: the moment is a parameter of the formation, and a test that leaves
///       it open would measure the clock and not the rule.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// The pieces a name of a file can be made of.
///
/// WHY: the pieces are drawn from a small set on purpose. A property test
///       over arbitrary text would draw names that no file system accepts,
///       and the formation would refuse them before it reached the property
///       that the test is about.
fn name_piece() -> impl Strategy<Value = String> {
    prop::sample::select(
        [
            "Duldung",
            "Leviathan",
            "Homegrown",
            "a",
            "bb",
            "ccc",
            "mixed Case",
            "Umlaut-Ärger",
            "Punkt.Name",
            "Bindestrich-Name",
        ]
        .iter()
        .map(|piece| (*piece).to_string())
        .collect::<Vec<String>>(),
    )
}

/// The name of a file, drawn at random.
fn file_name() -> impl Strategy<Value = String> {
    (
        name_piece(),
        prop::sample::select(
            [".avi", ".mkv", ".txt"]
                .iter()
                .map(|extension| (*extension).to_string())
                .collect::<Vec<String>>(),
        ),
    )
        .prop_map(|(stem, extension)| format!("{stem}{extension}"))
}

/// The rule, drawn at random.
fn rule() -> impl Strategy<Value = RenameRule> {
    prop::sample::select(
        [
            "[name].[ext]",
            "[name] - [n].[ext]",
            "[upper]_[lower].[ext]",
            "[parent]-[name].[ext]",
            "[rand]-[name].[ext]",
            "[name].[unknown].[ext]",
            "[n] - [name]",
        ]
        .iter()
        .map(|format| (*format).to_string())
        .collect::<Vec<String>>(),
    )
    .prop_map(|format| {
        let rule = RenameRule::new(format.clone()).with_counter(CounterRule::new(1, 1, 3));
        match format.contains(' ') {
            true => rule.with_replacement(ReplaceRule::literal(" ", "_")),
            false => rule,
        }
    })
}

/// The selection of the files, drawn at random.
fn selection() -> impl Strategy<Value = Vec<RenameInput>> {
    prop::collection::vec(file_name(), 0..6).prop_map(|names| {
        names
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                RenameInput::new(format!("/tv/The Expanse/{name}")).with_size(index as u64 * 100)
            })
            .collect()
    })
}

proptest! {
    /// The same rule and the same selection always yield the same names.
    ///
    /// WHY: this is FR-33 in its own words. The test draws the rule and the
    ///       selection at random, forms the names twice and compares them.
    #[test]
    fn the_formation_is_a_pure_function(rule in rule(), selection in selection()) {
        let first = form_names(&rule, &selection, moment());
        let second = form_names(&rule, &selection, moment());

        match (first, second) {
            (Ok(first), Ok(second)) => {
                prop_assert_eq!(first, second, "the same input yielded two different results");
            }
            (Err(first), Err(second)) => {
                prop_assert_eq!(
                    first.to_string(),
                    second.to_string(),
                    "the same input was refused twice in two different ways"
                );
            }
            (first, second) => {
                prop_assert!(
                    false,
                    "the same input was refused once and accepted once: {:?} and {:?}",
                    first.err(),
                    second.err()
                );
            }
        }
    }

    /// The value that stands for a random number depends on the name alone.
    ///
    /// WHY: the value has to be derived from the name of the file, and from
    ///       nothing else. Two selections that carry the same name at
    ///       different places have to give that name the same value — a
    ///       value that depended on the position would make the result
    ///       depend on the order of the selection.
    #[test]
    fn the_value_that_stands_for_a_random_number_depends_on_the_name_alone(
        first in file_name(),
        second in file_name(),
    ) {
        let rule = RenameRule::new("[rand]");
        let moment = moment();

        let alone = form_names(
            &rule,
            &[RenameInput::new(format!("/tv/{first}"))],
            moment,
        );
        let in_a_selection = form_names(
            &rule,
            &[
                RenameInput::new(format!("/tv/{second}")),
                RenameInput::new(format!("/tv/{first}")),
            ],
            moment,
        );

        let alone = names_of(alone);
        let in_a_selection = names_of(in_a_selection);

        // WHY: the value is looked up by the source of the entry, and not by
        //       its place in the list. The place changes when the selection
        //       changes, and the value may not.
        let path_of = |name: &str| format!("/tv/{name}");
        let value_of = |names: &[FormedName], path: &str| {
            names
                .iter()
                .find(|name| name.source() == std::path::Path::new(path))
                .and_then(|name| name.name().map(str::to_string))
                .unwrap_or_default()
        };

        prop_assert_eq!(
            value_of(&alone, &path_of(&first)),
            value_of(&in_a_selection, &path_of(&first)),
            "the value of the same name differed with its position"
        );
    }
}

/// The names of a result, or an empty list when the formation was refused.
fn names_of(
    result: Result<Vec<FormedName>, phocommander_ops::template::TemplateError>,
) -> Vec<FormedName> {
    result.unwrap_or_default()
}
