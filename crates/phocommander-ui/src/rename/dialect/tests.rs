//! The tests of the two dialects of the placeholders.
//!
//! WHAT: the test proves that the dialect of the dialog and the dialect of
//!       the named templates are two, and that the conversion between them
//!       touches only the placeholders the engine knows.
//! WHY: the specification decides in E13 that the rename engine knows both
//! dialects. A file of the named templates is read by the user, and the
//! dialect of that file is part of its interface.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --lib rename::dialect::tests
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::Dialect;
use crate::rename::{NamedTemplate, RenameRule};

/// The names that the engine knows.
const KNOWN: [&str; 4] = ["name", "n", "ext", "season"];

#[test]
fn a_placeholder_of_the_dialog_becomes_one_of_the_templates() {
    // act
    let converted = Dialect::convert("[name]_[n].[ext]", &KNOWN, Dialect::Square, Dialect::Curly);

    // assert
    assert_eq!(converted, "{name}_{n}.{ext}");
}

#[test]
fn a_placeholder_of_the_templates_becomes_one_of_the_dialog() {
    // act
    let converted = Dialect::convert(
        "{name} S{season}E{n}",
        &KNOWN,
        Dialect::Curly,
        Dialect::Square,
    );

    // assert
    assert_eq!(converted, "[name] S[season]E[n]");
}

#[test]
fn a_bracket_that_is_no_placeholder_stays_as_it_is() {
    // act
    // WHY: a file name may carry a bracket of its own. `episode [1].mkv`
    //       is a name, and the bracket is not a placeholder.
    let converted = Dialect::convert(
        "Folge [1] - [name]",
        &KNOWN,
        Dialect::Square,
        Dialect::Curly,
    );

    // assert
    assert_eq!(converted, "Folge [1] - {name}");
}

#[test]
fn a_bracket_that_is_no_placeholder_stays_as_it_is_in_the_other_way() {
    // act
    let converted = Dialect::convert(
        "Folge {1} - {name}",
        &KNOWN,
        Dialect::Curly,
        Dialect::Square,
    );

    // assert
    assert_eq!(converted, "Folge {1} - [name]");
}

#[test]
fn a_bracket_without_a_partner_stays_as_it_is() {
    // act
    // WHY: the first bracket finds the partner of the second one, and the
    //       name between them is no placeholder. The text therefore stays
    //       as it is — exactly as the evaluation of the placeholders keeps
    //       it.
    let converted = Dialect::convert("[name [1]", &KNOWN, Dialect::Square, Dialect::Curly);

    // assert
    assert_eq!(converted, "[name [1]");
}

#[test]
fn a_text_without_a_placeholder_is_not_changed() {
    // act
    let converted = Dialect::convert("just a name", &KNOWN, Dialect::Square, Dialect::Curly);

    // assert
    assert_eq!(converted, "just a name");
}

#[test]
fn the_conversion_into_the_same_dialect_changes_nothing() {
    // act
    let converted = Dialect::convert("[name]", &KNOWN, Dialect::Square, Dialect::Square);

    // assert
    assert_eq!(converted, "[name]");
}

#[test]
fn a_template_carries_its_format_in_the_dialect_of_the_file() {
    // arrange
    let template = NamedTemplate::new("Camera", RenameRule::new("[name]_[n].[ext]"));

    // act
    let in_file = template.in_dialect(Dialect::Square, Dialect::Curly, &KNOWN);
    let in_dialog = template.in_dialect(Dialect::Square, Dialect::Square, &KNOWN);

    // assert
    assert_eq!(in_file.rule.format, "{name}_{n}.{ext}");
    // WHY: the dialog keeps its own dialect, and the name of the template
    //       is the same in both.
    assert_eq!(in_dialog.rule.format, "[name]_[n].[ext]");
    assert_eq!(in_file.name, "Camera");
}

#[test]
fn a_template_survives_the_round_trip_through_both_dialects() {
    // arrange
    let template = NamedTemplate::new("Series", RenameRule::new("{name} S{season}E{n}"));
    let known = KNOWN.to_vec();

    // act
    let in_dialog = template.in_dialect(Dialect::Curly, Dialect::Square, &known);
    let back = in_dialog.in_dialect(Dialect::Square, Dialect::Curly, &known);

    // assert
    assert_eq!(back.rule.format, template.rule.format);
}
