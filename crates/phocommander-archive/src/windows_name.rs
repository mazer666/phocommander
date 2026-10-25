//! The file name rules of Windows, applied to the names an archive carries.
//!
//! WHAT: the characters that Windows forbids in the name of a file, the
//!       endings that Windows removes silently, and the function that turns
//!       such a name into one that every platform accepts.
//! WHY: an archive can come from anywhere. A name that is valid on the
//!       machine that extracts it can be invalid on Windows — where it is
//!       either refused or, worse, redirected to a device. The rules of this
//!       module are therefore stricter than the file system of the machine
//!       that happens to unpack the archive.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::windows_name::suggest_replacement;
//!
//! // The suggestion is offered to the user, never applied silently.
//! assert_eq!(suggest_replacement("report?.txt"), "report_.txt");
//! ```

/// The characters that Windows forbids in the name of a file.
///
/// WHY: {<}, {>}, {:}, {"}, {|}, {?} and {*} are the characters that the
///       Windows file system rejects. The two separators {/} and {\} are not
///       part of this list: they split a name into segments, and the segments
///       are checked by the resolution of the name itself.
pub const FORBIDDEN_CHARACTERS: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];

/// Returns the first character of the segment that Windows forbids.
///
/// WHY: the character is named in the error message, because a user who
///       wants to repair the name has to know which character is the problem.
pub fn first_forbidden_character(segment: &str) -> Option<char> {
    segment
        .chars()
        .find(|character| FORBIDDEN_CHARACTERS.contains(character))
}

/// Whether the segment ends in a dot or in a space.
///
/// WHY: Windows removes such an ending silently. A file that is called
///       {report.} is stored as {report}, so two entries of an archive that
///       differ only in that ending would fight for one file.
pub fn has_forbidden_ending(segment: &str) -> bool {
    segment.ends_with('.') || segment.ends_with(' ')
}

/// Returns the name with every character that Windows forbids replaced.
///
/// WHY: the suggestion is offered to the user instead of being applied. A
///       name that is changed silently produces a file that the archive did
///       not contain, and the user cannot tell which file it was. The
///       function is pure: the same name always produces the same
///       suggestion.
pub fn suggest_replacement(name: &str) -> String {
    let mut segments: Vec<String> = Vec::new();
    for segment in name.split(['/', '\\']) {
        let replaced: String = segment
            .chars()
            .map(|character| {
                if FORBIDDEN_CHARACTERS.contains(&character) {
                    '_'
                } else {
                    character
                }
            })
            .collect();
        let trimmed = replaced.trim_end_matches(['.', ' ']);
        let mut suggestion = trimmed.to_string();
        if suggestion.is_empty() && !segment.is_empty() {
            // The segment consisted only of dots, spaces or forbidden
            // characters. Windows has no name for it, so every character of
            // the segment becomes an underscore.
            suggestion = segment.chars().map(|_| '_').collect();
        }
        segments.push(suggestion);
    }
    segments.join("/")
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn finds_the_first_forbidden_character() {
        // act
        let found = first_forbidden_character("report?final.txt");

        // assert
        assert_eq!(found, Some('?'));
        assert_eq!(first_forbidden_character("report.txt"), None);
    }

    #[test]
    fn recognises_the_forbidden_endings() {
        // assert
        assert!(has_forbidden_ending("report."));
        assert!(has_forbidden_ending("report "));
        assert!(!has_forbidden_ending("report.txt"));
    }

    #[test]
    fn suggests_a_name_without_the_forbidden_character() {
        // act
        let suggestion = suggest_replacement("report?final.txt");

        // assert
        assert_eq!(suggestion, "report_final.txt");
        assert_eq!(first_forbidden_character(&suggestion), None);
    }

    #[test]
    fn suggests_a_name_without_the_forbidden_ending() {
        // act
        let suggestion = suggest_replacement("report.");

        // assert
        assert_eq!(suggestion, "report");
        assert!(!has_forbidden_ending(&suggestion));
    }

    #[test]
    fn suggests_a_name_for_a_segment_of_dots_only() {
        // act
        let suggestion = suggest_replacement("...");

        // assert
        assert_eq!(suggestion, "___");
    }

    #[test]
    fn the_suggestion_is_the_same_for_the_same_name() {
        // act
        let first = suggest_replacement("a/b?c.txt");
        let second = suggest_replacement("a/b?c.txt");

        // assert — the function is pure, the report of a refusal is stable
        assert_eq!(first, second);
    }

    #[test]
    fn keeps_the_directories_of_the_name() {
        // act
        let suggestion = suggest_replacement("docs/2026/report?.txt");

        // assert
        assert_eq!(suggestion, "docs/2026/report_.txt");
    }
}
