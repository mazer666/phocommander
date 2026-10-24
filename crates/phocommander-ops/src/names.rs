//! The rules for a name that is valid on every target platform.
//!
//! WHAT: the characters that Windows forbids
//!       ([{WINDOWS_FORBIDDEN_CHARACTERS}]), the characters that are
//!       forbidden on every platform ([{FORBIDDEN_CONTROL_CHARACTERS}]), the
//!       replacement that is suggested for a character
//!       ([{suggest_replacement}]), the characters of a name that are not
//!       allowed ([{invalid_characters}]) and the device names that Windows
//!       reserves ([{is_reserved_device_name}]).
//! WHY: the specification asks for the characters that the target platform
//!       forbids to be reported *before* the rename, together with a
//!       replacement that is suggested (FR-40). Nothing may be changed
//!       silently — the user decides. The rules live in one place, so that
//!       the rename dialog and the operations that write a file agree about
//!       which names are usable.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::names::{invalid_characters, suggest_replacement};
//!
//! let problems = invalid_characters("what?.jpg");
//! assert_eq!(problems.len(), 1);
//! assert_eq!(problems[0].0, '?');
//! assert_eq!(problems[0].1, suggest_replacement('?'));
//! ```

/// The characters that Windows does not allow in a file name.
///
/// WHY: a name that Windows refuses cannot be created there, and a
/// synchronisation between a Linux disk and a Windows disk has to notice
/// that before it starts, not after it copied a thousand files.
pub const WINDOWS_FORBIDDEN_CHARACTERS: [char; 9] =
    ['<', '>', ':', '"', '/', '\\', '?', '*', '|'];

/// The characters that no file system allows in a file name.
///
/// WHY: a control character ends the name for every system that reads it,
/// and a name that carries one cannot be shown in a list of files.
pub const FORBIDDEN_CONTROL_CHARACTERS: [char; 5] = ['\0', '\n', '\r', '\t', '\u{1b}'];

/// The replacement that is suggested for a character.
const REPLACEMENTS: [(char, char); 9] = [
    ('<', '('),
    ('>', ')'),
    (':', '-'),
    ('"', '\''),
    ('/', '-'),
    ('\\', '-'),
    ('?', '_'),
    ('*', '_'),
    ('|', '-'),
];

/// The replacement that is suggested for a character.
///
/// WHY: the suggestion has to be a character that every platform accepts, so
/// that the name the user confirms can really be written.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::names::suggest_replacement;
///
/// assert_eq!(suggest_replacement(':'), '-');
/// assert_eq!(suggest_replacement('?'), '_');
/// ```
pub fn suggest_replacement(character: char) -> char {
    for (forbidden, replacement) in REPLACEMENTS {
        if forbidden == character {
            return replacement;
        }
    }
    '_'
}

/// The characters of a name that are not allowed, with their replacement.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::names::invalid_characters;
///
/// assert!(invalid_characters("sunset.jpg").is_empty());
/// assert_eq!(invalid_characters("a:b").len(), 1);
/// ```
pub fn invalid_characters(name: &str) -> Vec<(char, char)> {
    let mut problems = Vec::new();
    for character in name.chars() {
        if WINDOWS_FORBIDDEN_CHARACTERS.contains(&character) {
            problems.push((character, suggest_replacement(character)));
            continue;
        }
        if character.is_control() {
            problems.push((character, '_'));
        }
    }
    problems
}

/// Whether the name is a device name that Windows reserves.
///
/// WHY: Windows refuses to create a file called {CON} or {NUL}, and it does
/// so in a way that silently redirects the write. A name that is reserved on
/// one platform has to be reported before the operation, not after it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::names::is_reserved_device_name;
///
/// assert!(is_reserved_device_name("CON"));
/// assert!(is_reserved_device_name("nul.txt"));
/// assert!(!is_reserved_device_name("concert.txt"));
/// ```
pub fn is_reserved_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") {
        return true;
    }

    match stem.as_bytes() {
        [b'C', b'O', b'M', digit] | [b'L', b'P', b'T', digit] => {
            matches!(digit, b'1'..=b'9')
        }
        _ => false,
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn the_forbidden_characters_are_reported_with_a_replacement() {
        // act
        let problems = invalid_characters("a:b?c");

        // assert
        assert_eq!(problems.len(), 2);
        assert_eq!(problems[0], (':', '-'));
        assert_eq!(problems[1], ('?', '_'));
    }

    #[test]
    fn a_name_without_a_problem_reports_nothing() {
        // act
        let problems = invalid_characters("S01E01 - The Expanse.mkv");

        // assert
        assert!(problems.is_empty());
    }

    #[test]
    fn a_control_character_is_reported() {
        // act
        let problems = invalid_characters("evil\u{0}.txt");

        // assert
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].1, '_');
    }

    #[test]
    fn every_forbidden_character_has_a_replacement_that_is_allowed() {
        // act
        let problems = invalid_characters(&WINDOWS_FORBIDDEN_CHARACTERS.iter().collect::<String>());

        // assert
        assert_eq!(problems.len(), WINDOWS_FORBIDDEN_CHARACTERS.len());
        for (character, replacement) in problems {
            assert!(!WINDOWS_FORBIDDEN_CHARACTERS.contains(&replacement), "{character}");
            assert!(!replacement.is_control(), "{character}");
        }
    }

    #[test]
    fn the_device_names_of_windows_are_recognised() {
        // act, assert
        assert!(is_reserved_device_name("CON"));
        assert!(is_reserved_device_name("nul.txt"));
        assert!(is_reserved_device_name("COM1"));
        assert!(!is_reserved_device_name("concert.txt"));
        assert!(!is_reserved_device_name("comics"));
    }
}
