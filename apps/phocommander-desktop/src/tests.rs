// The tests of the composition root of the desktop build.
//
// WHAT: the tests of the pieces that put the application together — the
//       parser of the key bindings and the builder of the application.
// WHY: the composition root is the one place where every crate of the
//       workspace meets. A mistake there is not caught by the tests of the
//       crates, because each of them is correct on its own.
// EXAMPLE:
// ```no_run
// use phocommander_desktop::parse_keymap;
//!
//! let tables = parse_keymap("[input.keyboard]\n\"panel.copy\" = \"F5\"\n");
//! assert_eq!(tables.len(), 1);
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

#[test]
fn the_parser_reads_the_bindings_of_every_context() {
    // arrange
    let text = "\
[input.keyboard]
\"panel.copy\" = \"F5\"
\"panel.rename\" = \"F2\"

[input.dpad]
\"panel.copy\" = \"palette:file.copy\"
";

    // act
    let tables = parse_keymap(text);

    // assert
    let tables = tables.expect("the table should be readable");
    assert_eq!(tables.len(), 2);
    assert_eq!(tables[&Context::Keyboard]["panel.copy"], "F5".to_string());
    assert_eq!(
        tables[&Context::Dpad]["panel.copy"],
        "palette:file.copy".to_string()
    );
}

#[test]
fn the_parser_leaves_the_comments_and_the_empty_lines_alone() {
    // arrange
    let text = "\
# A comment about the keyboard.
[input.keyboard]

\"panel.copy\" = \"F5\"
";

    // act
    let tables = parse_keymap(text);

    // assert
    let tables = tables.expect("the table should be readable");
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[&Context::Keyboard].len(), 1);
}

#[test]
fn a_file_without_bindings_carries_no_table() {
    // arrange
    let text = "# Only a comment.\n";

    // act
    let tables = parse_keymap(text);

    // assert
    assert!(tables.is_none(), "an empty file carries no table");
}

#[test]
fn the_application_is_built_from_the_safe_configuration() {
    // arrange
    let config = Config::safe();

    // act
    let app = build_application(&config);

    // assert
    // WHY: the application exists and can be drawn. The test does not open
    //       a window, because a window needs a screen and a test may run on
    //       a machine without one.
    let _ = app;
}

#[test]
fn the_translator_follows_the_language_of_the_configuration() {
    // arrange
    let mut config = Config::safe();

    // act
    config.general.language = Language::De;
    let german = build_translator(&config);

    // act
    config.general.language = Language::En;
    let english = build_translator(&config);

    // assert
    assert_eq!(german.locale().as_str(), "de");
    assert_eq!(english.locale().as_str(), "en");
}
