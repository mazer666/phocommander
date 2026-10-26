//! The translation layer of the work surface.
//!
//! WHAT: the translation keys the work surface uses ([`TextKey`]), the
//!       source of the translated texts ([`Catalogue`]) and the arguments a
//!       message may carry ([`Argument`]).
//! WHY: every visible text of the interface goes through a translation key
//!       (FR-05). The surface therefore never holds a literal: it asks the
//!       catalogue for a key, and the answer is never empty — a missing
//!       translation falls back to English and then to the key itself
//!       (FR-122).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::{Catalogue, TextKey, Texts};
//!
//! # struct German;
//! # impl Catalogue for German {
//! #     fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
//! #         None
//! #     }
//! # }
//! use phocommander_ui::Argument;
//!
//! let catalogue = German;
//! let texts = Texts::new(&catalogue);
//! assert_eq!(texts.get(TextKey::CommonCancel), "common-cancel");
//! ```

mod keys;

pub use keys::TextKey;

/// An argument of a translated message.
///
/// WHY: a file manager needs plural forms ("1 entry" against "2 entries").
/// The count is therefore an argument of the message and not a part of the
/// text, so that the plural rule stays in the catalogue. The catalogue of
/// the product reads the number as the argument `count` and the total as
/// the argument `total`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Argument<'a> {
    /// A number with the name that the message expects — `{ $count }` in the
    /// catalogue, for example.
    ///
    /// WHY: the name belongs to the argument and not to the caller of the
    ///       catalogue. The message of the catalogue decides which name a
    ///       value carries, and the catalogue of the product translates the
    ///       name into the placeholder of its own language. An argument
    ///       without a name would have to be guessed by the adapter, and the
    ///       guess would be wrong as soon as a message carries two numbers.
    Number {
        /// The name that the message expects.
        name: &'a str,

        /// The value.
        value: u64,
    },

    /// A text with the name that the message expects.
    Text {
        /// The name that the message expects.
        name: &'a str,

        /// The value.
        value: &'a str,
    },
}

impl<'a> Argument<'a> {
    /// The number of entries of a message.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Argument;
    ///
    /// let argument = Argument::count(7);
    /// assert_eq!(argument.name(), "count");
    /// ```
    pub fn count(value: u64) -> Self {
        Self::Number {
            name: "count",
            value,
        }
    }

    /// The number that a message compares its count with.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Argument;
    ///
    /// let argument = Argument::total(9);
    /// assert_eq!(argument.name(), "total");
    /// ```
    pub fn total(value: u64) -> Self {
        Self::Number {
            name: "total",
            value,
        }
    }

    /// A text with the name that the message expects.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Argument;
    ///
    /// let argument = Argument::named("side", "left");
    /// assert_eq!(argument.name(), "side");
    /// ```
    pub fn named(name: &'a str, value: &'a str) -> Self {
        Self::Text { name, value }
    }

    /// The name that the message expects.
    pub fn name(&self) -> &'a str {
        match self {
            Self::Number { name, .. } => name,
            Self::Text { name, .. } => name,
        }
    }

    /// The value of the argument.
    pub fn value(&self) -> ArgumentValue<'a> {
        match self {
            Self::Number { value, .. } => ArgumentValue::Number(*value),
            Self::Text { value, .. } => ArgumentValue::Text(value),
        }
    }
}

/// The value of an argument of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentValue<'a> {
    /// A number.
    Number(u64),

    /// A text.
    Text(&'a str),
}

/// The source of the translated texts of the interface.
///
/// WHY: the surface must not know how a catalogue is stored — that is the
/// business of the localisation module. The trait is the seam between the
/// two, and it is implemented by the shell that owns the catalogue.
pub trait Catalogue {
    /// The text of a key with the given arguments, or `None` when the
    /// catalogue does not know the key.
    fn lookup_with(&self, key: &str, arguments: &[Argument<'_>]) -> Option<String>;

    /// The text of a key without arguments.
    fn lookup(&self, key: &str) -> Option<String> {
        self.lookup_with(key, &[])
    }
}

/// The texts of the interface, read from a catalogue.
///
/// WHY: the helper owns the promise of the project that a visible text is
/// never empty. A catalogue that does not know a key is answered with the
/// key itself, which is visible and can be diagnosed.
#[derive(Clone, Copy)]
pub struct Texts<'a> {
    catalogue: &'a dyn Catalogue,
}

impl<'a> Texts<'a> {
    /// Reads the texts from a catalogue.
    pub fn new(catalogue: &'a dyn Catalogue) -> Self {
        Self { catalogue }
    }

    /// The text of a key, or the key itself when no catalogue knows it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::{Catalogue, TextKey, Texts};
    /// use phocommander_ui::Argument;
    ///
    /// # struct Empty;
    /// # impl Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    ///
    /// let texts = Texts::new(&Empty);
    /// assert_eq!(texts.get(TextKey::CommonOk), "common-ok");
    /// ```
    pub fn get(&self, key: TextKey) -> String {
        self.get_with(key, &[])
    }

    /// The text of a key with the given arguments, or the key itself.
    pub fn get_with(&self, key: TextKey, arguments: &[Argument<'_>]) -> String {
        match self.catalogue.lookup_with(key.as_str(), arguments) {
            Some(text) if !text.is_empty() => text,
            _ => key.as_str().to_string(),
        }
    }

    /// The text of a key that the catalogue names by its own name.
    ///
    /// WHY: the failures of the operations travel as a key and not as a
    ///       value of the enumeration of the keys, because the interface
    ///       knows no enumeration of the codes of the operations. The text is
    ///       read by the name of the key, and a key that the catalogue does
    ///       not know leaves its own name visible — a message that is empty
    ///       cannot be diagnosed (FR-122).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::i18n::{Argument, Catalogue, Texts};
    ///
    /// struct Empty;
    /// impl Catalogue for Empty {
    ///     fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
    ///         None
    ///     }
    /// }
    ///
    /// let texts = Texts::new(&Empty);
    /// assert_eq!(texts.get_by_key("path-too-long", &[]), "path-too-long");
    /// ```
    pub fn get_by_key(&self, key: &str, arguments: &[Argument<'_>]) -> String {
        match self.catalogue.lookup_with(key, arguments) {
            Some(text) if !text.is_empty() => text,
            _ => key.to_string(),
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// A catalogue that knows nothing — the worst case of the fallback.
    struct Empty;

    impl Catalogue for Empty {
        fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
            None
        }
    }

    /// A catalogue that answers every key with the key itself.
    struct Echo;

    impl Catalogue for Echo {
        fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
            Some(key.to_string())
        }
    }

    #[test]
    fn every_key_has_a_unique_name() {
        // act
        let names: BTreeSet<&str> = TextKey::all().iter().map(TextKey::as_str).collect();

        // assert
        assert_eq!(names.len(), TextKey::all().len());
    }

    #[test]
    fn a_missing_translation_falls_back_to_the_key() {
        // arrange
        let texts = Texts::new(&Empty);

        // act
        let text = texts.get(TextKey::CommonCancel);

        // assert
        assert_eq!(text, "common-cancel");
        assert!(!text.is_empty());
    }

    #[test]
    fn an_empty_translation_falls_back_to_the_key() {
        // WHY: a catalogue that answers with an empty string would draw an
        // empty label, which cannot be diagnosed.
        // arrange
        struct Blank;
        impl Catalogue for Blank {
            fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
                Some(String::new())
            }
        }
        let texts = Texts::new(&Blank);

        // act
        let text = texts.get(TextKey::CommonCancel);

        // assert
        assert_eq!(text, "common-cancel");
    }

    #[test]
    fn the_catalogue_is_asked_with_the_arguments() {
        // arrange
        struct Counting {
            calls: std::cell::Cell<usize>,
        }
        impl Catalogue for Counting {
            fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
                self.calls.set(self.calls.get() + 1);
                Some("7 entries".to_string())
            }
        }
        let counting = Counting {
            calls: std::cell::Cell::new(0),
        };
        let texts = Texts::new(&counting);

        // act
        let text = texts.get_with(TextKey::PanelStatusEntries, &[Argument::count(7)]);

        // assert
        assert_eq!(text, "7 entries");
        assert_eq!(counting.calls.get(), 1);
    }

    #[test]
    fn the_text_of_a_catalogue_is_used_as_it_is() {
        // arrange
        let texts = Texts::new(&Echo);

        // act
        let text = texts.get(TextKey::PanelColumnName);

        // assert
        assert_eq!(text, "panel-column-name");
    }
}
