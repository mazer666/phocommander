//! The locales of the interface.
//!
//! WHAT: a language of the interface ([`Locale`]) and the chain of the lookup
//!       that a requested locale produces ([`lookup_chain`]).
//! WHY: the catalogue of a language lives in a directory of its own, so that
//!       adding a language is a directory and not a change in the source code
//!       (FR-125). The locales are therefore read from the embedded catalogue
//!       at run time and are not an enumeration in the code. The chain of the
//!       lookup starts here: requested locale, language fallback chain,
//!       English, the key itself (section 8.3 of the specification).
//! EXAMPLE:
//! ```no_run
//! use phocommander_i18n::Locale;
//!
//! assert_eq!(Locale::parse("de").map(|locale| locale.as_str().to_string()),
//!            Some("de".to_string()));
//! assert!(Locale::all().iter().any(|locale| locale.as_str() == "en"));
//! ```

use unic_langid::LanguageIdentifier;

use crate::catalogue::catalogue_directories;

/// The directory that holds the English catalogue.
///
/// WHY: English is the last step of the lookup chain before the key itself
/// (section 8.3 of the specification), and every other language falls back
/// to it.
const ENGLISH: &str = "en";

/// A language of the interface.
///
/// WHY: the locales are read from the embedded catalogue at run time and are
/// not an enumeration in the code. A language that is added as a directory
/// below `locales/` is therefore known without a change in the source code
/// (FR-125).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Locale {
    /// The name of the catalogue directory, for example `en` or `de`.
    directory: String,

    /// The identifier of the language, as the catalogue reads it.
    identifier: LanguageIdentifier,
}

impl Locale {
    /// Every locale of the product, in a stable order.
    ///
    /// WHY: the order is fixed so that a list of locales — in the settings,
    /// in the documentation or in a test — always reads the same way.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::Locale;
    ///
    /// let locales = Locale::all();
    /// let names: Vec<&str> = locales.iter().map(Locale::as_str).collect();
    /// assert!(names.contains(&"en"));
    /// ```
    pub fn all() -> Vec<Locale> {
        catalogue_directories()
            .into_iter()
            .map(|directory| Locale::of(&directory))
            .collect()
    }

    /// The locale of a catalogue directory.
    ///
    /// WHY: the name of the directory is the name of the locale. A directory
    /// whose name is not a language identifier is still usable — the catalogue
    /// reads it as the unknown language — so that a broken directory produces
    /// a readable message instead of a panic.
    pub fn of(directory: &str) -> Locale {
        Locale {
            directory: directory.to_string(),
            identifier: identifier_of(directory),
        }
    }

    /// English — the last step of the lookup chain before the key itself.
    pub fn english() -> Locale {
        Locale::of(ENGLISH)
    }

    /// The locale with the given spelling, if a catalogue holds it.
    ///
    /// WHY: the configuration file names the locale, and a regional variant
    /// such as `de-AT` belongs to the language `de`. The lookup therefore
    /// compares the language part and accepts the variant.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::Locale;
    ///
    /// assert_eq!(Locale::parse("de-AT").map(|locale| locale.as_str().to_string()),
    ///            Some("de".to_string()));
    /// ```
    pub fn parse(raw: &str) -> Option<Self> {
        let wanted = normalise(raw);
        let directories = catalogue_directories();

        if directories.contains(&wanted) {
            return Some(Locale::of(&wanted));
        }

        let language = wanted.split('-').next().unwrap_or(wanted.as_str());
        if directories.contains(language) {
            return Some(Locale::of(language));
        }

        None
    }

    /// The locale the operating system asks for.
    ///
    /// WHY: the language of the interface follows the system, and the user
    /// can change it in the settings afterwards (FR-124). A system that
    /// reports no locale at all is not an error — English is the fallback.
    pub fn from_system() -> Self {
        match sys_locale::get_locale() {
            Some(raw) => Locale::parse(&raw).unwrap_or_else(Locale::english),
            None => Locale::english(),
        }
    }

    /// The spelling of the locale in the catalogue directory and in the
    /// configuration file.
    pub fn as_str(&self) -> &str {
        &self.directory
    }

    /// The identifier of the language, as the catalogue reads it.
    pub fn identifier(&self) -> &LanguageIdentifier {
        &self.identifier
    }

    /// Whether this locale is English.
    pub fn is_english(&self) -> bool {
        self.directory == ENGLISH
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The chain of the lookup for a requested locale.
///
/// WHY: the chain of the specification (section 8.3) is "requested locale →
/// language fallback chain → English → the key itself". The chain is built
/// here, so that the catalogue itself does not have to know how a fallback
/// works. A locale that no catalogue holds leads to English, and a key that
/// English does not know leads to the key itself.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_i18n::lookup_chain;
///
/// let chain = lookup_chain("de-AT");
/// let spellings: Vec<String> = chain.iter().map(|locale| locale.as_str().to_string()).collect();
/// assert_eq!(spellings, vec!["de".to_string(), "en".to_string()]);
/// ```
pub fn lookup_chain(requested: &str) -> Vec<Locale> {
    let directories = catalogue_directories();
    let wanted = normalise(requested);
    let mut chain: Vec<Locale> = Vec::new();

    // The requested locale itself, when a catalogue holds it.
    if directories.contains(&wanted) {
        chain.push(Locale::of(&wanted));
    }

    // The language fallback chain: the language of the requested locale, then
    // the language without the region, and so on. "de-AT" therefore leads to
    // "de", and "zh-Hant-TW" leads to "zh-Hant" and then to "zh".
    let mut candidate = wanted.as_str();
    while let Some((language, _)) = candidate.split_once('-') {
        if directories.contains(language) && !chain.iter().any(|locale| locale.as_str() == language)
        {
            chain.push(Locale::of(language));
        }
        candidate = language;
    }

    // English is the last step before the key itself.
    if !chain.iter().any(Locale::is_english) {
        chain.push(Locale::english());
    }

    chain
}

/// The spelling of a locale in the form the catalogue directories use.
fn normalise(raw: &str) -> String {
    raw.trim().replace('_', "-").to_ascii_lowercase()
}

/// The identifier of the language of a catalogue directory.
///
/// WHY: a directory that carries no valid language tag is not a crash and not
///       a silent wrong language: the identifier of the unknown language
///       ("und") is the tag that CLDR itself uses for it. The catalogue of
///       such a directory is then empty, and the lookup walks on to the next
///       step of its chain.
fn identifier_of(directory: &str) -> LanguageIdentifier {
    match directory.parse::<LanguageIdentifier>() {
        Ok(identifier) => identifier,
        Err(_) => LanguageIdentifier::from_bytes(b"und").unwrap_or_else(|_| {
            // WHY: the tag "und" is a valid tag of the standard, so the
            //       parser cannot refuse it. The fallback exists only so that
            //       the function has no way to panic.
            LanguageIdentifier::from_parts("und".parse().unwrap_or_default(), None, None, &[])
        }),
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn every_locale_has_a_unique_spelling() {
        // act
        let locales = Locale::all();
        let spellings: Vec<&str> = locales.iter().map(Locale::as_str).collect();

        // assert
        let mut sorted = spellings.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), spellings.len());
    }

    #[test]
    fn reads_a_locale_by_its_spelling() {
        // act
        let found = Locale::parse("de");

        // assert
        assert_eq!(
            found.map(|locale| locale.as_str().to_string()),
            Some("de".into())
        );
        assert_eq!(Locale::parse("xx"), None);
    }

    #[test]
    fn accepts_a_regional_variant_of_a_known_language() {
        // act
        let found = Locale::parse("de-AT");

        // assert
        assert_eq!(
            found.map(|locale| locale.as_str().to_string()),
            Some("de".into())
        );
    }

    #[test]
    fn the_english_catalogue_is_always_part_of_the_product() {
        // act
        let locales = Locale::all();

        // assert
        assert!(locales.iter().any(Locale::is_english));
    }

    #[test]
    fn the_system_locale_is_one_of_the_shipped_locales() {
        // act
        let locale = Locale::from_system();

        // assert
        assert!(Locale::all().contains(&locale));
    }

    #[test]
    fn the_chain_of_a_known_language_leads_to_english() {
        // act
        let chain: Vec<String> = lookup_chain("de")
            .iter()
            .map(|locale| locale.as_str().to_string())
            .collect();

        // assert
        assert_eq!(chain, vec!["de".to_string(), "en".to_string()]);
    }

    #[test]
    fn the_chain_of_a_regional_variant_leads_to_the_language() {
        // act
        let chain: Vec<String> = lookup_chain("de-AT")
            .iter()
            .map(|locale| locale.as_str().to_string())
            .collect();

        // assert
        assert_eq!(chain, vec!["de".to_string(), "en".to_string()]);
    }

    #[test]
    fn the_chain_of_an_unknown_language_leads_to_english() {
        // act
        let chain: Vec<String> = lookup_chain("xx")
            .iter()
            .map(|locale| locale.as_str().to_string())
            .collect();

        // assert
        assert_eq!(chain, vec!["en".to_string()]);
    }

    #[test]
    fn the_chain_of_english_is_english() {
        // act
        let chain: Vec<String> = lookup_chain("en-US")
            .iter()
            .map(|locale| locale.as_str().to_string())
            .collect();

        // assert
        assert_eq!(chain, vec!["en".to_string()]);
    }
}
