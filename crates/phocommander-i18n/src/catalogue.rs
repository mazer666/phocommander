// The catalogues and the lookup chain of Phocommander.
// The files under `locales/` are the shipped translations.
//
// WHAT: the embedded Fluent catalogues ([`Translator`]), the directories that
//       hold them ([`catalogue_directories`]) and the reasons why a catalogue
//       could not be read ([`CatalogueError`]).
// WHY: the interface is translated, and a missing translation must never
//       produce an empty label: the lookup walks "requested locale → language
//       fallback chain → English → the key itself" (FR-122). The catalogues
//       are embedded into the binary, so that the program works on a machine
//       that has no configuration directory at all.
// EXAMPLE:
// ```no_run
// use phocommander_i18n::{Locale, Translator};
//
// # fn main() -> Result<(), phocommander_i18n::CatalogueError> {
// let translator = Translator::new(Locale::of("de"))?;
// assert_eq!(translator.text("common-ok"), "OK");
// # Ok(())
// # }
// ```

use std::collections::BTreeSet;

use fluent::{FluentArgs, FluentBundle, FluentResource};
use fluent_bundle::types::FluentNumber;
use fluent_syntax::ast::Entry;
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

use crate::format::LocaleFormat;
use crate::locale::{Locale, lookup_chain};

/// The embedded catalogue directory.
#[derive(RustEmbed)]
#[folder = "locales/"]
struct Locales;

/// The domains of the catalogue, one file per domain.
///
/// WHY: the catalogue is split by domain so that a translator can work on the
/// file manager without touching the viewer or the settings (section 8.2 of
/// the specification).
const DOMAINS: [&str; 6] = ["common", "actions", "panel", "viewer", "palette", "rename"];

/// The directories that hold the embedded catalogues.
///
/// WHY: the locales of the product are the directories of the catalogue and
/// not a list in the code. A language that is added as a directory is
/// therefore known without a change in the source code (FR-125).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_i18n::catalogue_directories;
///
/// assert!(catalogue_directories().contains("en"));
/// ```
pub fn catalogue_directories() -> BTreeSet<String> {
    Locales::iter()
        .filter_map(|path| path.split('/').next().map(str::to_string))
        .collect()
}

/// The catalogue of one locale and the lookup that reads it.
///
/// WHY: the translator owns the fallback chain, so that no caller has to
/// remember it. A key that is missing in the requested locale is looked up in
/// the next locale of the chain, and a key that is missing everywhere is
/// answered with the key itself — never with an empty string.
// WHY: the derive of Debug cannot be used here: the bundle of Fluent does not
//       implement Debug, and the translator holds a list of bundles. The
//       fields that carry the state are written by hand instead, so that the
//       translator can be printed in a log without the texts of the catalogue.
/// A value that a message of the catalogue carries.
///
/// WHY: the value is either a number or a text, and the two are written
///       differently. A number that is handed over as a text loses the
///       separators of the locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageValue {
    /// A number.
    Number(u64),

    /// A text.
    Text(String),
}

/// The catalogues of the locales of the interface.
///
/// WHY: the translator owns the catalogues of the chain of the lookup and the
///       rules of the language of the first locale. It is the one place where
///       the texts of the product and the rules of the language meet, so that
///       neither the interface nor the application has to know how a
///       catalogue is stored (FR-122, FR-123).
pub struct Translator {
    chain: Vec<Locale>,
    bundles: Vec<FluentBundle<FluentResource>>,
    keys: BTreeSet<String>,

    /// The formats of the language of the translator.
    ///
    /// WHY: Fluent writes a number without the separators of the language —
    ///       "1234" in German as well as in English. The rules of the
    ///       language live in the data of CLDR, and the translator owns them
    ///       so that the numbers of the interface carry the separators of the
    ///       language (FR-123). The formats are missing when the data of the
    ///       language does not carry them; the number then stands without a
    ///       separator, and the interface is still readable.
    formats: Option<LocaleFormat>,
}

impl std::fmt::Debug for Translator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Translator")
            .field("chain", &self.chain)
            .field("keys", &self.keys)
            .finish()
    }
}

impl Translator {
    /// Reads the catalogue of a locale.
    ///
    /// WHY: a catalogue that cannot be read is a broken installation and not
    /// a runtime condition. The error therefore names the file and the
    /// reason, so that the person who maintains the catalogue can fix it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::{Locale, Translator};
    ///
    /// # fn main() -> Result<(), phocommander_i18n::CatalogueError> {
    /// let translator = Translator::new(Locale::english())?;
    /// assert!(translator.has("common-ok"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(locale: Locale) -> Result<Self, CatalogueError> {
        let mut chain = vec![locale];
        if !chain[0].is_english() {
            chain.push(Locale::english());
        }
        Self::from_chain(chain)
    }

    /// Reads the catalogue of a requested locale.
    ///
    /// WHY: the requested locale may name a regional variant (`de-AT`) or a
    /// language that the product does not ship. The chain of the lookup
    /// answers both cases: the language of the request, then English, then
    /// the key itself.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::Translator;
    ///
    /// # fn main() -> Result<(), phocommander_i18n::CatalogueError> {
    /// let translator = Translator::for_requested("de-AT")?;
    /// assert_eq!(translator.text("common-cancel"), "Abbrechen");
    /// # Ok(())
    /// # }
    /// ```
    pub fn for_requested(requested: &str) -> Result<Self, CatalogueError> {
        Self::from_chain(lookup_chain(requested))
    }

    /// The locale of this catalogue.
    pub fn locale(&self) -> &Locale {
        &self.chain[0]
    }

    /// The chain of locales that the lookup walks.
    ///
    /// WHY: the chain is the evidence of the promise of the specification
    /// (section 8.3). A test reads it and knows which step answers a key.
    pub fn chain(&self) -> &[Locale] {
        &self.chain
    }

    /// The text of a key, or the key itself when no catalogue knows it.
    ///
    /// WHY: an empty label in the interface cannot be diagnosed, a visible
    /// key can (FR-122). The lookup therefore never returns an empty string.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::{Locale, Translator};
    ///
    /// # fn main() -> Result<(), phocommander_i18n::CatalogueError> {
    /// let translator = Translator::new(Locale::english())?;
    /// assert_eq!(translator.text("a-key-that-does-not-exist"), "a-key-that-does-not-exist");
    /// # Ok(())
    /// # }
    /// ```
    pub fn text(&self, key: &str) -> String {
        self.text_with_args(key, &FluentArgs::new())
    }

    /// The text of a key with the arguments of the message.
    ///
    /// WHY: a file manager needs plural forms ("1 entry" against "2
    /// entries"), and Fluent carries them natively (section 8.2).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use fluent::FluentArgs;
    /// use phocommander_i18n::{Locale, Translator};
    ///
    /// # fn main() -> Result<(), phocommander_i18n::CatalogueError> {
    /// let translator = Translator::new(Locale::english())?;
    /// let mut args = FluentArgs::new();
    /// args.set("count", 2);
    /// assert_eq!(translator.text_with_args("panel-status-entries", &args), "2 entries");
    /// # Ok(())
    /// # }
    /// ```
    pub fn text_with_args(&self, key: &str, args: &FluentArgs) -> String {
        for bundle in &self.bundles {
            if let Some(text) = format_message(bundle, key, args) {
                return text;
            }
        }

        key.to_string()
    }

    /// The message of a key with the given arguments.
    ///
    /// WHY: a message of the catalogue can carry values — a number of
    ///       entries, the name of a panel. The values carry the name that
    ///       the message expects, and the catalogue of the product turns
    ///       that name into the placeholder of its own language. A number is
    ///       handed over as a number and not as a text, so that the catalogue
    ///       writes it with the separators of the locale — "1.234" in German
    ///       and "1,234" in English (FR-123).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::{Locale, MessageValue, Translator};
    ///
    /// let translator = Translator::new(Locale::english())
    ///     .unwrap_or_else(|error| panic!("the catalogue should be readable: {error}"));
    /// let text = translator.text_with_arguments(
    ///     "panel-status-entries",
    ///     &[("count".to_string(), MessageValue::Number(7))],
    /// );
    /// assert!(text.contains('7'));
    /// ```
    pub fn text_with_arguments(&self, key: &str, arguments: &[(String, MessageValue)]) -> String {
        let mut values = FluentArgs::new();
        for (name, value) in arguments {
            match value {
                MessageValue::Number(number) => {
                    // WHY: the number is written twice, and each time for a
                    //       reason. The text carries the separators of the
                    //       language — "1.234" in German and "1,234" in
                    //       English — and the raw number is what the plural
                    //       rule of the message selects with. A catalogue
                    //       that shows a number therefore writes the name of
                    //       the text for the display and the name of the
                    //       raw number for the selection:
                    //       `{ $count_number -> [one] … *[other] { $count } … }`.
                    let text = match &self.formats {
                        Some(formats) => formats.number(*number),
                        None => number.to_string(),
                    };
                    values.set(name.as_str(), text);
                    // WHY: the name is handed over as an owned string. The
                    //       arguments own it, and it outlives the call.
                    values.set(format!("{name}_number"), FluentNumber::from(*number));
                }
                MessageValue::Text(text) => {
                    values.set(name.as_str(), text.as_str());
                }
            }
        }
        self.text_with_args(key, &values)
    }

    /// Whether the catalogue of this locale knows the key.
    ///
    /// WHY: the test that compares the key sets of two locales needs to know
    /// which locale holds a key, not only the text of the key.
    pub fn has(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    /// The keys of the catalogue of this locale.
    pub fn keys(&self) -> &BTreeSet<String> {
        &self.keys
    }

    /// Builds a translator that walks a chain of locales.
    fn from_chain(chain: Vec<Locale>) -> Result<Self, CatalogueError> {
        let mut bundles = Vec::new();
        let mut keys = BTreeSet::new();

        for (position, locale) in chain.iter().enumerate() {
            let (bundle, locale_keys) = build_bundle(locale.clone())?;
            if position == 0 {
                keys = locale_keys;
            }
            bundles.push(bundle);
        }

        // WHY: the formats belong to the language of the translator — the
        //       first locale of the chain. A locale that the data of CLDR
        //       does not know falls back to English, which is the fallback of
        //       the whole product.
        let formats = LocaleFormat::for_tag(chain[0].as_str()).ok();

        Ok(Self {
            chain,
            bundles,
            keys,
            formats,
        })
    }
}

/// The keys of the catalogue of a locale.
///
/// WHY: the gate of the specification compares the key sets of the locales
/// (section 8.5). The function reads them from the embedded catalogue, so
/// that the gate cannot pass while a file is missing.
pub fn catalogue_keys(locale: Locale) -> Result<BTreeSet<String>, CatalogueError> {
    Ok(build_bundle(locale)?.1)
}

/// The keys of every catalogue of the product.
///
/// WHY: the gate of the specification walks every locale directory. The
/// function answers with the keys of every catalogue, so that the gate does
/// not have to know how the catalogues are stored.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_i18n::catalogue_keys_of_every_locale;
///
/// let keysets = catalogue_keys_of_every_locale();
/// assert!(!keysets.is_empty());
/// ```
pub fn catalogue_keys_of_every_locale() -> Vec<(String, BTreeSet<String>)> {
    Locale::all()
        .into_iter()
        .filter_map(|locale| match catalogue_keys(locale.clone()) {
            Ok(keys) => Some((locale.as_str().to_string(), keys)),
            Err(_) => None,
        })
        .collect()
}

/// Builds the bundle of a locale from the embedded files.
fn build_bundle(
    locale: Locale,
) -> Result<(FluentBundle<FluentResource>, BTreeSet<String>), CatalogueError> {
    let language: LanguageIdentifier = locale.identifier().clone();

    let mut bundle = FluentBundle::new(vec![language]);
    // WHY: the isolation marks of Fluent are invisible characters around
    // every placeholder. They are correct for a bidirectional text and wrong
    // for a file manager that compares a translated label with a key.
    bundle.set_use_isolating(false);
    // WHY: the built-in functions of Fluent are not registered by the
    //       constructor. Without this call the message `NUMBER($count)` is
    //       an unknown function, the message falls back to the key, and the
    //       numbers of the interface carry no separator of the locale.
    bundle
        .add_builtins()
        .map_err(|error| CatalogueError::Rejected {
            path: format!("the built-in functions of the locale {locale}"),
            problems: vec![error.to_string()],
        })?;

    let mut identifiers = BTreeSet::new();

    for domain in DOMAINS {
        let path = format!("{}/{domain}.ftl", locale.as_str());
        let file =
            Locales::get(&path).ok_or(CatalogueError::MissingDomain { path: path.clone() })?;
        let source = String::from_utf8_lossy(&file.data).into_owned();

        let resource = FluentResource::try_new(source).map_err(|(_, errors)| {
            CatalogueError::InvalidSyntax {
                path: path.clone(),
                problems: errors
                    .iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<String>>(),
            }
        })?;

        for entry in resource.entries() {
            if let Entry::Message(message) = entry {
                identifiers.insert(message.id.name.to_string());
            }
        }

        bundle
            .add_resource(resource)
            .map_err(|errors| CatalogueError::Rejected {
                path: path.clone(),
                problems: errors
                    .iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<String>>(),
            })?;
    }

    Ok((bundle, identifiers))
}

/// Formats the message of a key, or {None} when the bundle does not hold it.
fn format_message(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: &FluentArgs,
) -> Option<String> {
    let message = bundle.get_message(key)?;
    let pattern = message.value()?;

    let mut errors = Vec::new();
    let value = bundle.format_pattern(pattern, Some(args), &mut errors);
    if !errors.is_empty() {
        // WHY: a message whose arguments do not fit is a broken message. The
        // lookup then continues with the fallback instead of showing a text
        // with error markers in it.
        return None;
    }

    Some(value.to_string())
}

/// The reasons why a catalogue could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogueError {
    /// The locale has no catalogue directory.
    #[error("the locale {locale} is not known")]
    UnknownLocale {
        /// The locale that was asked for.
        locale: String,
    },

    /// A domain file is missing from the catalogue.
    #[error("the catalogue file {path} is missing")]
    MissingDomain {
        /// The file that is missing.
        path: String,
    },

    /// A domain file could not be parsed.
    #[error("the catalogue file {path} could not be parsed: {problems:?}")]
    InvalidSyntax {
        /// The file that could not be parsed.
        path: String,

        /// The problems the parser reported.
        problems: Vec<String>,
    },

    /// A domain file was rejected by the bundle.
    #[error("the catalogue file {path} was rejected: {problems:?}")]
    Rejected {
        /// The file that was rejected.
        path: String,

        /// The problems the bundle reported.
        problems: Vec<String>,
    },
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule that
// forbids a panic applies to the production code, and it is enforced there.
mod tests {
    use super::*;

    /// The translator of a locale, or a panic with the reason.
    fn translator(directory: &str) -> Translator {
        let locale = Locale::of(directory);
        match Translator::new(locale) {
            Ok(translator) => translator,
            Err(error) => panic!("the catalogue should be readable: {error}"),
        }
    }

    #[test]
    fn a_number_that_carries_a_value_is_written_with_the_separators_of_the_locale() {
        // act
        let german = translator("de").text_with_arguments(
            "panel-status-entries",
            &[("count".to_string(), MessageValue::Number(1_234))],
        );
        let english = translator("en").text_with_arguments(
            "panel-status-entries",
            &[("count".to_string(), MessageValue::Number(1_234))],
        );
        let one = translator("en").text_with_arguments(
            "panel-status-entries",
            &[("count".to_string(), MessageValue::Number(1))],
        );

        // assert
        // WHY: the number is handed over as a number and not as a text. The
        //       catalogue therefore writes it with the separators of its own
        //       language, and the German one carries the point where the
        //       English one carries the comma (FR-123).
        assert!(
            german.contains("1.234"),
            "the German catalogue should write the point: {german}"
        );
        assert!(
            english.contains("1,234"),
            "the English catalogue should write the comma: {english}"
        );
        // WHY: the plural rule selects with the raw number, and the text
        //       shows the number of the language.
        assert_eq!(one, "1 entry");
    }

    #[test]
    fn a_text_that_carries_a_value_reaches_the_message() {
        // act
        let text = translator("en").text_with_arguments(
            "error-transfer-same-panel",
            &[("side".to_string(), MessageValue::Text("left".to_string()))],
        );

        // assert
        assert!(
            text.contains("left"),
            "the message should carry the value: {text}"
        );
    }
}
