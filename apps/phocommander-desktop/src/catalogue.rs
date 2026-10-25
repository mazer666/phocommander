//! The catalogue of the interface of the desktop build.
//!
//! WHAT: the adapter that connects the catalogue of the crate
//!       `phocommander-i18n` with the interface of the crate
//!       `phocommander-ui` ([`TranslatorCatalogue`]), the adapter that
//!       connects the formats of a locale with the same interface
//!       ([`LocaleFormats`]) and the catalogue that knows no key at all
//!       ([`EmptyCatalogue`]).
//! WHY: the interface asks for a [`Catalogue`] and for [`Formats`], and the
//!       catalogue of the product knows its texts and its formats in its
//!       own way. The adapters are the one place where the two meet, so
//!       that neither crate has to know the other.
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::catalogue::TranslatorCatalogue;
//! use phocommander_i18n::{Locale, Translator};
//! use phocommander_ui::Catalogue;
//!
//! let translator = Translator::new(Locale::english())
//!     .unwrap_or_else(|error| panic!("the catalogue should be readable: {error}"));
//! let catalogue = TranslatorCatalogue::new(translator);
//! assert!(catalogue.lookup("panel-column-name").is_some());
//! ```

use std::time::SystemTime;

use phocommander_i18n::{Locale, LocaleFormat, MessageValue, Translator};
use phocommander_ui::{Argument, ArgumentValue, Catalogue, Formats, PlainFormats};

/// The catalogue of the interface, backed by the translations of the product.
///
/// WHY: the interface asks for the text of a key and for nothing else. The
///       catalogue of the product answers with the text of a key and never
///       with an empty string — a missing translation falls back to English
///       and then to the key itself (FR-122). The adapter therefore hands
///       the answer of the translator on unchanged, and it answers with
///       `None` only when the catalogue knows the key not at all, which lets
///       the interface fall back to the key.
#[derive(Debug)]
pub struct TranslatorCatalogue {
    /// The translator that holds the catalogues of the locales.
    translator: Translator,
}

impl TranslatorCatalogue {
    /// Builds the catalogue of the interface.
    pub fn new(translator: Translator) -> Self {
        Self { translator }
    }

    /// The translator that holds the catalogues.
    pub fn translator(&self) -> &Translator {
        &self.translator
    }
}

impl Catalogue for TranslatorCatalogue {
    fn lookup_with(&self, key: &str, arguments: &[Argument<'_>]) -> Option<String> {
        // WHY: the translator answers with the key itself when no catalogue
        //       knows it. That answer is the right one for the interface, and
        //       it is not an empty text. The adapter reports therefore only
        //       the keys that the catalogue really knows.
        if !self.translator.has(key) {
            return None;
        }

        // WHY: the arguments carry the name that the message expects, and the
        //       name is what the catalogue of the product turns into the
        //       placeholder of its own language. A number is handed over as
        //       a number and not as a text, so that the catalogue writes it
        //       with the separators of the locale — "1.234" in German and
        //       "1,234" in English (FR-123).
        let values: Vec<(String, MessageValue)> = arguments
            .iter()
            .map(|argument| {
                let value = match argument.value() {
                    ArgumentValue::Number(number) => MessageValue::Number(number),
                    ArgumentValue::Text(text) => MessageValue::Text(text.to_string()),
                };
                (argument.name().to_string(), value)
            })
            .collect();

        Some(self.translator.text_with_arguments(key, &values))
    }
}

/// The formats of the interface, backed by the rules of a locale.
///
/// WHY: the numbers, the sizes, the dates and the times of the interface
///       follow the rules of the locale of the user (FR-123). Those rules
///       live in the data of CLDR, and the crate of the localisation owns
///       that data. The adapter is therefore the one place where the two
///       meet, so that the interface crate may not depend on the crate of
///       the localisation (ADR-0001, option B).
#[derive(Debug, Clone)]
pub struct LocaleFormats {
    /// The formats of the locale.
    format: LocaleFormat,
}

impl LocaleFormats {
    /// Builds the formats of a locale.
    pub fn of(format: LocaleFormat) -> Self {
        Self { format }
    }

    /// The formats of the locale that these formats carry.
    pub fn format(&self) -> &LocaleFormat {
        &self.format
    }
}

impl Formats for LocaleFormats {
    fn number(&self, value: u64) -> String {
        self.format.number(value)
    }

    fn size(&self, bytes: u64) -> String {
        self.format.size(bytes)
    }

    fn date(&self, moment: SystemTime) -> String {
        self.format.date(moment)
    }

    fn time(&self, moment: SystemTime) -> String {
        self.format.time(moment)
    }

    fn date_time(&self, moment: SystemTime) -> String {
        self.format.date_time(moment)
    }
}

/// The catalogue that knows no key at all.
///
/// WHY: the interface falls back to the key itself when no catalogue knows
///       it, and the key is visible and can be diagnosed (FR-122). This
///       catalogue is the answer of a broken installation: the program
///       starts, the labels show their keys, and the program does not stop.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyCatalogue;

impl Catalogue for EmptyCatalogue {
    fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        None
    }
}

/// The formats of a locale, or the plain formats when the locale carries no
/// data.
///
/// WHY: a locale that the data of CLDR does not know is not a crash and not
///       an empty text: the plain formats are written instead, and the
///       interface stays readable. The plain formats are the formats of the
///       product before the locales were part of it.
pub fn formats_for(tag: &str) -> Box<dyn Formats> {
    match LocaleFormat::for_tag(tag) {
        Ok(format) => Box::new(LocaleFormats::of(format)),
        Err(_) => Box::new(PlainFormats),
    }
}

/// The catalogue of a locale, or the empty catalogue when the catalogue
/// cannot be read.
///
/// WHY: a catalogue that cannot be read is a broken installation and not a
///       reason to stop the program. The empty catalogue lets the interface
///       show its keys, which are visible and can be diagnosed (FR-122).
pub fn catalogue_for(locale: &Locale) -> Box<dyn Catalogue> {
    match Translator::new(locale.clone()) {
        Ok(translator) => Box::new(TranslatorCatalogue::new(translator)),
        Err(_) => Box::new(EmptyCatalogue),
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_formats_of_a_german_locale_write_the_sizes_the_way_german_writes_them() {
        // WHY: FR-123 — the size of a file follows the locale of the user.
        //       The test reads the separator of the decimals out of the text.
        // act
        let formats = formats_for("de");

        // assert
        assert_eq!(formats.size(1_200_000), "1,2 MB");
    }

    #[test]
    fn the_formats_of_an_english_locale_write_the_sizes_the_way_english_writes_them() {
        // act
        let formats = formats_for("en");

        // assert
        assert_eq!(formats.size(1_200_000), "1.2 MB");
    }

    #[test]
    fn a_locale_without_data_falls_back_to_the_plain_formats() {
        // WHY: a broken locale must not stop the program and must not leave
        //       an empty text behind.
        // act
        let formats = formats_for("not a tag");

        // assert
        assert_eq!(formats.size(2_048), "2.0 KiB");
    }

    #[test]
    fn the_catalogue_of_a_known_locale_knows_the_keys_of_the_product() {
        // act
        let catalogue = catalogue_for(&Locale::of("de"));

        // assert
        assert!(catalogue.lookup("panel-column-name").is_some());
    }

    #[test]
    fn the_catalogue_of_an_unknown_locale_knows_no_key() {
        // act
        let catalogue = catalogue_for(&Locale::of("xx"));

        // assert
        assert!(catalogue.lookup("panel-column-name").is_none());
    }
}
