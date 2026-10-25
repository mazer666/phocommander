//! The tests of the formation of the new names.
//!
//! WHY: a test may fail loudly when its arrangement is wrong. The rule that
//!       forbids a panic applies to the production code, and it is enforced
//!       there.

use std::time::{Duration, SystemTime};

use super::*;

#[test]
fn the_placeholders_are_replaced_by_their_values() {
    // arrange
    let rule = RenameRule::new("[name]|[ext]|[parent]|[date]|[time]|[size]|[hash]");
    let input = RenameInput::new("/photos/Duldung.avi")
        .with_size(4096)
        .with_checksum(0x1234_5678_9abc_def0);
    let now = moment();

    // act
    let names = form_names(&rule, &[input], now);

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(
        formed[0].name(),
        Some("Duldung|avi|photos|2023-11-14|22-13-20|4096|12345678")
    );
}

#[test]
fn the_case_placeholders_write_the_name_in_another_case() {
    // arrange
    let rule = RenameRule::new("[upper]|[lower]|[case]|[title]");
    let input = RenameInput::new("/photos/the expanse.mkv");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(
        formed[0].name(),
        Some("THE EXPANSE|the expanse|The expanse|The Expanse")
    );
}

#[test]
fn a_placeholder_that_is_not_known_stays_in_the_name() {
    // arrange
    let rule = RenameRule::new("[name].[unknown].[ext]");
    let input = RenameInput::new("/photos/notes[1].txt");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("notes[1].[unknown].txt"));
}

#[test]
fn the_random_value_is_the_same_for_the_same_name() {
    // arrange
    let rule = RenameRule::new("[rand]");
    let input = RenameInput::new("/photos/Duldung.avi");

    // act
    let first = form_names(&rule, std::slice::from_ref(&input), moment());
    let second = form_names(&rule, &[input], moment());

    // assert
    let first = match first {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    let second = match second {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    let value = first[0].name().unwrap_or_default().to_string();
    assert_eq!(value.len(), 8);
    assert_eq!(value, second[0].name().unwrap_or_default());
}

#[test]
fn the_caller_may_hand_in_the_random_value() {
    // arrange
    let rule = RenameRule::new("[rand]");
    let input = RenameInput::new("/photos/Duldung.avi").with_random("a1b2c3d4");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("a1b2c3d4"));
}

#[test]
fn the_search_replaces_the_text_literally() {
    // arrange
    let rule = RenameRule::new("[name]").with_replacement(ReplaceRule::literal("a.b", "c"));
    let input = RenameInput::new("/photos/axb.txt");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    // WHY: the dot of the search text is a dot and not "any character".
    assert_eq!(formed[0].name(), Some("axb"));
}

#[test]
fn the_search_replaces_the_text_with_a_regular_expression() {
    // arrange
    let rule = RenameRule::new("[name]").with_replacement(ReplaceRule::pattern(r"^a(\d+)$", "b$1"));
    let input = RenameInput::new("/photos/a42.txt");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("b42"));
}

#[test]
fn the_dot_of_a_pattern_matches_any_character() {
    // arrange
    let rule = RenameRule::new("[name]").with_replacement(ReplaceRule::pattern("a.b", "c"));
    let input = RenameInput::new("/photos/axb.txt");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("c"));
}

#[test]
fn a_pattern_that_cannot_be_read_stops_the_formation() {
    // arrange
    let rule = RenameRule::new("[name]").with_replacement(ReplaceRule::pattern("a(", "b"));
    let input = RenameInput::new("/photos/a42.txt");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let error = match names {
        Ok(_) => panic!("the broken pattern should have been refused"),
        Err(error) => error,
    };
    assert!(matches!(error, TemplateError::InvalidPattern { .. }));
}

#[test]
fn the_counter_numbers_the_files_of_the_selection() {
    // arrange
    let rule = RenameRule::new("[n]").with_counter(CounterRule::new(1, 1, 3));
    let inputs = vec![
        RenameInput::new("/photos/a.jpg"),
        RenameInput::new("/photos/b.jpg"),
        RenameInput::new("/photos/c.jpg"),
    ];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("001"));
    assert_eq!(formed[1].name(), Some("002"));
    assert_eq!(formed[2].name(), Some("003"));
}

#[test]
fn the_counter_can_count_by_the_name() {
    // arrange
    let rule = RenameRule::new("[n] [name]")
        .with_counter(CounterRule::new(1, 1, 1).with_order(CounterOrder::Name));
    let inputs = vec![
        RenameInput::new("/photos/c.jpg"),
        RenameInput::new("/photos/a.jpg"),
        RenameInput::new("/photos/b.jpg"),
    ];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("3 c"));
    assert_eq!(formed[1].name(), Some("1 a"));
    assert_eq!(formed[2].name(), Some("2 b"));
}

#[test]
fn the_counter_is_written_after_the_replacement() {
    // arrange
    let rule = RenameRule::new("[n]")
        .with_counter(CounterRule::new(1, 1, 3))
        .with_replacement(ReplaceRule::literal("0", "O"));
    let input = RenameInput::new("/photos/a.jpg");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("001"));
}

#[test]
fn a_format_that_carries_the_counter_gets_the_standard_counter() {
    // arrange
    // WHY: the user writes "[n]" and expects a number. A name that carries
    //       the text "[n]" would be a surprise in every file of the
    //       selection, and the standard counter is what the rule of the
    //       camera template uses.
    let rule = RenameRule::new("[n] - [name]");
    let input = RenameInput::new("/photos/a.jpg");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("001 - a"));
}

#[test]
fn a_format_without_the_counter_placeholder_gets_no_counter() {
    // arrange
    // WHY: a format that does not ask for a number gets none. The counter
    //       would only cost time.
    let rule = RenameRule::new("[name].[ext]");
    let input = RenameInput::new("/photos/a.jpg");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("a.jpg"));
}

#[test]
fn a_value_that_the_caller_did_not_supply_is_reported() {
    // arrange
    let rule = RenameRule::new("[hash]");
    let input = RenameInput::new("/photos/a.jpg");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert!(formed[0].name().is_none());
    assert!(matches!(
        formed[0].problem(),
        Some(TemplateError::UnknownValue {
            placeholder: "hash",
            ..
        })
    ));
}

#[test]
fn a_rule_without_a_format_is_refused() {
    // arrange
    let rule = RenameRule::new("   ");
    let input = RenameInput::new("/photos/a.jpg");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    assert!(matches!(names, Err(TemplateError::EmptyFormat)));
}

#[test]
fn a_name_with_a_separator_is_refused() {
    // arrange
    let rule = RenameRule::new("[parent]/[name]");
    let input = RenameInput::new("/photos/a.jpg");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert!(formed[0].name().is_none());
    assert!(matches!(
        formed[0].problem(),
        Some(TemplateError::InvalidName { .. })
    ));
}

#[test]
fn the_series_reads_the_numbers_out_of_the_name() {
    // arrange
    let rule = RenameRule::new("").with_series(SeriesRule::new());
    let inputs = vec![RenameInput::new("/tv/The Expanse/The.Expanse.S01E03.avi")];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(
        formed[0].name(),
        Some("The Expanse - S01E03 - The.Expanse.avi")
    );
}

#[test]
fn the_series_name_is_the_name_of_the_parent_directory() {
    // arrange
    let rule = RenameRule::new("").with_series(SeriesRule::new().with_season(1).with_episode(1));
    let input = RenameInput::new("/tv/The Expanse/episode.avi");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("The Expanse - S01E01 - episode.avi"));
}

#[test]
fn the_series_name_can_be_overridden() {
    // arrange
    let rule = RenameRule::new("").with_series(
        SeriesRule::new()
            .with_series_name("Duldung")
            .with_season(1)
            .with_episode(1),
    );
    let input = RenameInput::new("/tv/Collection/episode.avi");

    // act
    let names = form_names(&rule, &[input], moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("Duldung - S01E01 - episode.avi"));
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
    let names = form_names(&rule, &inputs, moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert_eq!(formed[0].name(), Some("The Expanse - S01E099 - a.avi"));
    assert_eq!(formed[1].name(), Some("The Expanse - S01E100 - b.avi"));
}

#[test]
fn the_same_rule_and_the_same_files_always_yield_the_same_names() {
    // arrange
    let rule =
        RenameRule::new("[rand] - [name] - [n] - [date]").with_counter(CounterRule::new(1, 1, 3));
    let inputs = vec![
        RenameInput::new("/photos/a.jpg"),
        RenameInput::new("/photos/b.jpg"),
    ];

    // act
    let first = form_names(&rule, &inputs, moment());
    let second = form_names(&rule, &inputs, moment());

    // assert
    let first = match first {
        Ok(formed) => formed,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    let second = match second {
        Ok(formed) => formed,
        Err(error) => panic!("the names should have been formed: {error}"),
    };
    assert_eq!(first, second);
}

#[test]
fn the_pairs_of_the_rename_are_built_from_the_names() {
    // arrange
    let rule = RenameRule::new("[name].bak");
    let inputs = vec![RenameInput::new("/photos/a.jpg")];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    let pairs = match rename_pairs(&formed) {
        Some(pairs) => pairs,
        None => panic!("the pairs should have been built"),
    };
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].target.to_string_lossy(), "/photos/a.bak");
}

#[test]
fn a_selection_with_a_file_that_could_not_be_named_has_no_pairs() {
    // arrange
    let rule = RenameRule::new("[hash]");
    let inputs = vec![RenameInput::new("/photos/a.jpg")];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let formed = match names {
        Ok(formed) => formed,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    assert!(rename_pairs(&formed).is_none());
}

#[test]
fn the_characters_that_the_platform_forbids_are_reported() {
    // act
    let problems = name_problems("A:B<C");

    // assert
    assert_eq!(problems.len(), 2);
    assert_eq!(problems[0], (':', '-'));
    assert_eq!(problems[1], ('<', '('));
}

/// The moment that every test uses.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}
