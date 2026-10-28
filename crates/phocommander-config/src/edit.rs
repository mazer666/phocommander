//! The editor of the configuration file.
//!
//! WHAT: the writing of one setting into the text of the configuration file
//!       ([`set_language`]).
//! WHY: the configuration file belongs to the user. It carries comments that
//!       explain every setting, and the order in which the user wrote them
//!       down. A program that rewrites the whole file destroys both, and a
//!       program that rewrites nothing cannot answer a choice of the user
//!       that happens while the program runs — the language of the interface
//!       for instance (FR-124).
//!       The editor therefore changes the one line that carries the setting
//!       and leaves everything else as it is.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::edit::set_language;
//! use phocommander_config::Language;
//!
//! let document = "[general]\nlanguage = \"de\"\n";
//! let changed = set_language(document, Language::En).unwrap_or_else(|error| panic!("the line should be found: {error}"));
//! assert_eq!(changed, "[general]\nlanguage = \"en\"\n");
//! ```

use crate::Language;

/// The error of the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// The section that carries the setting is not in the file.
    MissingSection {
        /// The name of the section.
        section: &'static str,
    },

    /// The file cannot be written.
    Io {
        /// The message of the system.
        message: String,
    },
}

impl std::fmt::Display for EditError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSection { section } => {
                write!(formatter, "the section [{section}] is not in the file")
            }
            Self::Io { message } => write!(formatter, "the file cannot be written: {message}"),
        }
    }
}

impl std::error::Error for EditError {}

/// Writes the language into the text of the configuration file.
///
/// WHY: the line that carries the setting is found by its name and its
///       section, and only that line is replaced. A comment above the line
///       stays above the line, and the order of the file stays the order the
///       user wrote. When the line is not there, it is inserted into the
///       section — at the end of it, so that it does not separate the
///       comment of the setting below from its setting.
pub fn set_language(document: &str, language: Language) -> Result<String, EditError> {
    let mut zeilen: Vec<String> = document.lines().map(str::to_string).collect();
    let mut in_general = false;
    let mut gefunden = false;

    for zeile in zeilen.iter_mut() {
        let getrimmt = zeile.trim();
        if getrimmt.starts_with('[') {
            in_general = getrimmt == "[general]";
            continue;
        }
        if !in_general {
            continue;
        }
        // WHY: the name of the setting comes before the equals sign, and the
        //       value behind it. A comment after the value belongs to the
        //       line and stays with it.
        let Some((name, rest)) = zeile.split_once('=') else {
            continue;
        };
        if name.trim() != "language" {
            continue;
        }
        let wert = format!("\"{}\"", language.as_str());
        let neu = match rest.find('#') {
            Some(stelle) => format!("{} = {wert}  {}", name.trim_end(), &rest[stelle..]),
            None => format!("{} = {wert}", name.trim_end()),
        };
        *zeile = neu;
        gefunden = true;
        break;
    }

    if !gefunden {
        // WHY: the section has to exist. A file without it does not name the
        //       language at all, and the program that writes one would
        //       silently create a section that the user never saw.
        if !zeilen.iter().any(|zeile| zeile.trim() == "[general]") {
            return Err(EditError::MissingSection { section: "general" });
        }
        let stelle = zeilen
            .iter()
            .position(|zeile| zeile.trim() == "[general]")
            .unwrap_or_default();
        // WHY: the setting goes behind the last line of the section, so that
        //       it does not push the comment of the setting below away. The
        //       empty lines at the end of the section belong to the section
        //       that follows it, and the setting goes before them.
        let mut ende = stelle + 1;
        while ende < zeilen.len() && !zeilen[ende].trim_start().starts_with('[') {
            ende += 1;
        }
        let mut einbau = ende;
        while einbau > stelle + 1 && zeilen[einbau - 1].trim().is_empty() {
            einbau -= 1;
        }
        zeilen.insert(einbau, format!("language = \"{}\"", language.as_str()));
    }

    let mut ergebnis = zeilen.join("\n");
    if document.ends_with('\n') {
        ergebnis.push('\n');
    }
    Ok(ergebnis)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_line_of_the_language_is_replaced() {
        // arrange
        let document = "[general]\nlanguage = \"de\"\nrestore_session = true\n";

        // act
        let ergebnis = set_language(document, Language::En)
            .unwrap_or_else(|error| panic!("the line should be found: {error}"));

        // assert
        assert_eq!(
            ergebnis,
            "[general]\nlanguage = \"en\"\nrestore_session = true\n"
        );
    }

    #[test]
    fn a_comment_of_the_line_stays_with_the_line() {
        // arrange
        let document = "[general]\nlanguage = \"de\"  # the language of the interface\n";

        // act
        let ergebnis = set_language(document, Language::En)
            .unwrap_or_else(|error| panic!("the line should be found: {error}"));

        // assert
        assert_eq!(
            ergebnis,
            "[general]\nlanguage = \"en\"  # the language of the interface\n"
        );
    }

    #[test]
    fn the_comments_of_the_file_stay_where_they_are() {
        // arrange
        let document =
            "# The settings of the user.\n[general]\n# The language.\nlanguage = \"de\"\n";

        // act
        let ergebnis = set_language(document, Language::En)
            .unwrap_or_else(|error| panic!("the line should be found: {error}"));

        // assert
        assert_eq!(
            ergebnis,
            "# The settings of the user.\n[general]\n# The language.\nlanguage = \"en\"\n"
        );
    }

    #[test]
    fn a_missing_line_is_inserted_at_the_end_of_the_section() {
        // arrange
        let document = "[general]\nrestore_session = true\n\n[panels]\nlayout = \"horizontal\"\n";

        // act
        let ergebnis = set_language(document, Language::En)
            .unwrap_or_else(|error| panic!("the section should be found: {error}"));

        // assert
        assert_eq!(
            ergebnis,
            "[general]\nrestore_session = true\nlanguage = \"en\"\n\n[panels]\nlayout = \"horizontal\"\n"
        );
    }

    #[test]
    fn a_file_without_the_section_is_refused() {
        // arrange
        let document = "[panels]\nlayout = \"horizontal\"\n";

        // act
        let ergebnis = set_language(document, Language::En);

        // assert
        assert_eq!(
            ergebnis,
            Err(EditError::MissingSection { section: "general" })
        );
    }

    #[test]
    fn a_line_of_another_section_is_not_touched() {
        // arrange
        let document = "[general]\nrestore_session = true\n\n[other]\nlanguage = \"de\"\n";

        // act
        let ergebnis = set_language(document, Language::En)
            .unwrap_or_else(|error| panic!("the section should be found: {error}"));

        // assert
        // WHY: the setting of the other section belongs to the other section.
        assert_eq!(
            ergebnis,
            "[general]\nrestore_session = true\nlanguage = \"en\"\n\n[other]\nlanguage = \"de\"\n"
        );
    }
}
