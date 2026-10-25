// The catalogues and the lookup chain of Phocommander.
// The files under `locales/` are the shipped translations.
//
// WHAT: the embedded Fluent catalogues ([`Translator`]) and the reasons why a
//       catalogue could not be read ([`CatalogueError`]).
// WHY: the interface is translated, and a missing translation must never
//       produce an empty label: the lookup walks "requested locale → English
//       → the key itself" (FR-122). The catalogues are embedded into the
//       binary, so that the program works on a machine that has no
//       configuration directory at all.
// EXAMPLE:
// ```no_run
// use phocommander_i18n::{Locale, Translator};
//
// # fn main() -> Result<(), phocommander_i18n::CatalogueError> {
// let translator = Translator::new(Locale::German)?;
// assert_eq!(translator.text("common-ok"), "OK");
// # Ok(())
// # }
// ```

use std::collections::BTreeSet;

use fluent::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::Entry;
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

use crate::locale::Locale;

/// The embedded catalogue directory.
#[derive(RustEmbed)]
#[folder = "locales/"]
struct Locales;

/// The domains of the catalogue, one file per domain.
///
/// WHY: the catalogue is split by domain so that a translator can work on the
/// file manager without touching the viewer or the settings (section 8.2 of
/// the specification).
const DOMAINS: [&str; 5] = ["common", "actions", "panel", "viewer", "palette"];

/// The catalogue of one locale and the lookup that reads it.
///
/// WHY: the translator owns the fallback chain, so that no caller has to
/// remember it. A key that is missing in the requested locale is looked up in
/// English, and a key that is missing everywhere is answered with the key
/// itself — never with an empty string.
pub struct Translator {
    locale: Locale,
    bundle: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
    keys: BTreeSet<String>,
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
    /// let translator = Translator::new(Locale::English)?;
    /// assert!(translator.has("common-ok"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(locale: Locale) -> Result<Self, CatalogueError> {
        let (bundle, keys) = build_bundle(locale)?;
        let fallback = match locale.fallback() {
            Some(fallback_locale) => Some(build_bundle(fallback_locale)?.0),
            None => None,
        };

        Ok(Self {
            locale,
            bundle,
            fallback,
            keys,
        })
    }

    /// The locale of this catalogue.
    pub fn locale(&self) -> Locale {
        self.locale
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
    /// let translator = Translator::new(Locale::English)?;
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
    /// let translator = Translator::new(Locale::English)?;
    /// let mut args = FluentArgs::new();
    /// args.set("count", 2);
    /// assert_eq!(translator.text_with_args("panel-status-entries", &args), "2 entries");
    /// # Ok(())
    /// # }
    /// ```
    pub fn text_with_args(&self, key: &str, args: &FluentArgs) -> String {
        if let Some(text) = format_message(&self.bundle, key, args) {
            return text;
        }

        if let Some(fallback) = &self.fallback
            && let Some(text) = format_message(fallback, key, args)
        {
            return text;
        }

        key.to_string()
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
}

/// The keys of the catalogue of a locale.
///
/// WHY: the gate of the specification compares the key sets of the locales
/// (section 8.5). The function reads them from the embedded catalogue, so
/// that the gate cannot pass while a file is missing.
pub fn catalogue_keys(locale: Locale) -> Result<BTreeSet<String>, CatalogueError> {
    Ok(build_bundle(locale)?.1)
}

/// Builds the bundle of a locale from the embedded files.
fn build_bundle(
    locale: Locale,
) -> Result<(FluentBundle<FluentResource>, BTreeSet<String>), CatalogueError> {
    let language: LanguageIdentifier =
        locale
            .as_str()
            .parse()
            .map_err(|_| CatalogueError::UnknownLocale {
                locale: locale.as_str().to_string(),
            })?;

    let mut bundle = FluentBundle::new(vec![language]);
    // WHY: the isolation marks of Fluent are invisible characters around
    // every placeholder. They are correct for a bidirectional text and wrong
    // for a file manager that compares a translated label with a key.
    bundle.set_use_isolating(false);

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
