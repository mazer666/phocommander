//! The locales of the interface.
//!
//! WHAT: the languages the interface ships in ([{Locale}]) and the way a
//!       locale is found ([{Locale::from_system}]).
//! WHY: the catalogue format is Fluent and the catalogue of a language lives
//!       in a directory of its own, so that adding a language needs no change
//!       in the source code (FR-125). The lookup chain of the project —
//!       requested locale, language fallback, English, the key itself —
//!       starts here.
//! EXAMPLE:
//! ```no_run
//! use phocommander_i18n::Locale;
//!
//! assert_eq!(Locale::German.as_str(), "de");
//! assert_eq!(Locale::parse("de"), Some(Locale::German));
//! ```

/// A language of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Locale {
    /// English — the language of the fallback chain and the language of the
    /// action identifiers, which never change with the interface language.
    English,

    /// German.
    German,
}

impl Locale {
    /// Every locale, in a stable order.
    ///
    /// WHY: the order is fixed so that a list of locales — in the settings,
    /// in the documentation or in a test — always reads the same way.
    pub const fn all() -> [Locale; 2] {
        [Locale::English, Locale::German]
    }

    /// The spelling of the locale in the catalogue directory and in the
    /// configuration file.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Locale::English => "en",
            Locale::German => "de",
        }
    }

    /// The locale with the given spelling.
    ///
    /// WHY: the configuration file names the locale, and a regional variant
    /// such as 'de-AT' belongs to the language {de}. The lookup therefore
    /// compares the language part and accepts the variant.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::Locale;
    ///
    /// assert_eq!(Locale::parse("de-AT"), Some(Locale::German));
    /// assert_eq!(Locale::parse("fr"), None);
    /// ```
    pub fn parse(raw: &str) -> Option<Self> {
        let language = raw
            .split(['-', '_'])
            .next()
            .unwrap_or(raw)
            .trim()
            .to_ascii_lowercase();
        match language.as_str() {
            "en" => Some(Locale::English),
            "de" => Some(Locale::German),
            _ => None,
        }
    }

    /// The locale that is used when a key is missing in this locale.
    ///
    /// WHY: the chain of the specification is "requested locale → language
    /// fallback chain → English → the key itself". English is the fallback
    /// of every other locale, and it is its own fallback.
    pub const fn fallback(&self) -> Option<Locale> {
        match self {
            Locale::English => None,
            Locale::German => Some(Locale::English),
        }
    }

    /// The locale the operating system asks for.
    ///
    /// WHY: the language of the interface follows the system, and the user
    /// can change it in the settings afterwards (FR-124). A system that
    /// reports no locale at all is not an error — English is the fallback.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_i18n::Locale;
    ///
    /// let locale = Locale::from_system();
    /// assert!(Locale::all().contains(&locale));
    /// ```
    pub fn from_system() -> Self {
        sys_locale::get_locale()
            .and_then(|locale| Locale::parse(&locale))
            .unwrap_or(Locale::English)
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
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
        let spellings: Vec<&str> = Locale::all().iter().map(Locale::as_str).collect();

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
        assert_eq!(found, Some(Locale::German));
        assert_eq!(Locale::parse("fr"), None);
    }

    #[test]
    fn accepts_a_regional_variant_of_a_known_language() {
        // act
        let found = Locale::parse("de-AT");

        // assert
        assert_eq!(found, Some(Locale::German));
    }

    #[test]
    fn english_is_the_fallback_of_every_other_locale() {
        // act
        let german = Locale::German.fallback();
        let english = Locale::English.fallback();

        // assert
        assert_eq!(german, Some(Locale::English));
        assert_eq!(english, None);
    }

    #[test]
    fn the_system_locale_is_one_of_the_shipped_locales() {
        // act
        let locale = Locale::from_system();

        // assert
        assert!(Locale::all().contains(&locale));
    }
}
