//! The catalogue of the interface of the desktop build.
//!
//! WHAT: the adapter that connects the catalogue of the crate
//!       `phocommander-i18n` with the interface of the crate
//!       `phocommander-ui` ([`TranslatorCatalogue`]).
//! WHY: the interface asks for a [`Catalogue`], and the catalogue of the
//!       product knows its texts in its own way. The adapter is the one place
//!       where the two meet, so that neither crate has to know the other.
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

use phocommander_i18n::Translator;
use phocommander_ui::{Argument, Catalogue};

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
    fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        // WHY: the translator answers with the key itself when no catalogue
        //       knows it. That answer is the right one for the interface, and
        //       it is not an empty text. The adapter reports therefore only
        //       the keys that the catalogue really knows.
        if self.translator.has(key) {
            Some(self.translator.text(key))
        } else {
            None
        }
    }
}
