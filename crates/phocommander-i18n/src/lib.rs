//! The localisation layer of Phocommander.
//!
//! WHAT: the locales of the interface ([{Locale}]), the embedded Fluent
//!       catalogues ([{Translator}]) and the reasons why a catalogue could
//!       not be read ([{CatalogueError}]).
//! WHY: every visible string of the product is externalised, so that the
//!       interface can be translated without touching the source code
//!       (FR-125) and so that a missing translation can never produce an
//!       empty label (FR-122). The lookup chain is fixed: requested locale →
//!       English → the key itself.
//! BOUNDARY: this crate owns the catalogue and the lookup. It knows nothing
//!       about widgets, panels or the file system, and it opens no file at
//!       run time — the catalogues are embedded into the binary.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_i18n::{Locale, Translator};
//!
//! # fn main() -> Result<(), phocommander_i18n::CatalogueError> {
//! let translator = Translator::new(Locale::German)?;
//! assert_eq!(translator.text("common-cancel"), "Abbrechen");
//! # Ok(())
//! # }
//! ```

/// The catalogues of the interface language.
///
/// WHY: every translated text lives in a catalogue, and the catalogue
/// itself knows nothing about the application. That keeps the translations
/// readable for a translator who does not read Rust.
pub mod catalogue;
pub mod locale;

pub use catalogue::{CatalogueError, Translator, catalogue_keys};
pub use locale::Locale;

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use fluent::FluentArgs;

    use super::*;

    /// The locales that ship with the product.
    fn locales() -> [Locale; 2] {
        Locale::all()
    }

    #[test]
    fn every_locale_holds_the_same_keys() {
        // WHY: FR-121 — the key set is identical in every language. A
        // translation that is missing is a defect that the gate of the
        // specification reports.
        // arrange
        let english = match catalogue_keys(Locale::English) {
            Ok(keys) => keys,
            Err(error) => panic!("the English catalogue should be readable: {error}"),
        };
        let german = match catalogue_keys(Locale::German) {
            Ok(keys) => keys,
            Err(error) => panic!("the German catalogue should be readable: {error}"),
        };

        // act
        let missing_in_german: Vec<&String> = english.difference(&german).collect();
        let missing_in_english: Vec<&String> = german.difference(&english).collect();

        // assert
        assert!(
            missing_in_german.is_empty(),
            "the German catalogue is missing: {missing_in_german:?}"
        );
        assert!(
            missing_in_english.is_empty(),
            "the English catalogue is missing: {missing_in_english:?}"
        );
        assert_eq!(english.len(), german.len());
    }

    #[test]
    fn every_locale_carries_a_translation_for_every_key() {
        // arrange
        let expected = match catalogue_keys(Locale::English) {
            Ok(keys) => keys,
            Err(error) => panic!("the English catalogue should be readable: {error}"),
        };

        // act
        for locale in locales() {
            let translator = match Translator::new(locale) {
                Ok(translator) => translator,
                Err(error) => panic!("the catalogue should be readable: {error}"),
            };

            for key in &expected {
                assert!(translator.has(key), "{locale} does not know the key {key}");
                assert!(
                    !translator.text(key).is_empty(),
                    "{locale} answers the key {key} with an empty text"
                );
            }
        }
    }

    #[test]
    fn a_missing_key_falls_back_to_the_key_itself() {
        // arrange
        let translator = match Translator::new(Locale::English) {
            Ok(translator) => translator,
            Err(error) => panic!("the catalogue should be readable: {error}"),
        };

        // act
        let text = translator.text("a-key-that-nobody-wrote");

        // assert
        assert_eq!(text, "a-key-that-nobody-wrote");
        assert!(!text.is_empty());
    }

    #[test]
    fn the_fallback_chain_reaches_the_english_text() {
        // WHY: FR-122 — a missing translation falls back to English and then
        // to the key, never to an empty string.
        // arrange
        let translator = match Translator::new(Locale::German) {
            Ok(translator) => translator,
            Err(error) => panic!("the catalogue should be readable: {error}"),
        };

        // act
        let text = translator.text("common-cancel");

        // assert
        assert_eq!(text, "Abbrechen");
    }

    #[test]
    fn the_plural_form_follows_the_number() {
        // arrange
        let translator = match Translator::new(Locale::English) {
            Ok(translator) => translator,
            Err(error) => panic!("the catalogue should be readable: {error}"),
        };
        let mut one = FluentArgs::new();
        one.set("count", 1);
        let mut many = FluentArgs::new();
        many.set("count", 7);

        // act
        let singular = translator.text_with_args("panel-status-entries", &one);
        let plural = translator.text_with_args("panel-status-entries", &many);

        // assert
        assert_eq!(singular, "1 entry");
        assert_eq!(plural, "7 entries");
    }
}
