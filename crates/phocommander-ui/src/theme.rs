//! The theme of Phocommander.
//!
//! WHAT: the colour scheme the user chose and the egui visuals that belong
//!       to it.
//! WHY: the theme is a setting of the user, but the colours that egui draws
//!       with are not. The translation from one to the other happens here, in
//!       the only crate that knows about egui, so that the rest of the
//!       application can talk about "dark" instead of about colour values.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::{visuals, Theme};
//!
//! assert!(visuals(Theme::Dark, false).dark_mode);
//! assert!(!visuals(Theme::Light, true).dark_mode);
//! ```

/// The colour scheme the user chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// The colour scheme of the operating system.
    #[default]
    System,

    /// The light colour scheme.
    Light,

    /// The dark colour scheme.
    Dark,
}

impl Theme {
    /// The spelling of this theme in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    /// The theme with the given spelling in the configuration file.
    ///
    /// WHY: the theme of the operating system is decided by the platform
    /// adapter, which knows the preference of the user. It arrives here as a
    /// name, so that this crate does not have to know the platform.
    pub fn from_config_name(name: &str) -> Option<Self> {
        match name {
            "system" => Some(Theme::System),
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

/// The egui visuals that belong to a theme.
///
/// WHY: the theme `system` follows the preference of the operating system,
/// which the caller reads. Every other theme ignores that preference, so
/// that a user who wants a light interface on a dark system gets one.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::{visuals, Theme};
///
/// assert!(visuals(Theme::System, true).dark_mode);
/// assert!(!visuals(Theme::System, false).dark_mode);
/// ```
pub fn visuals(theme: Theme, system_prefers_dark: bool) -> egui::Visuals {
    match theme {
        Theme::System => {
            if system_prefers_dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            }
        }
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_theme_by_its_configuration_name() {
        // act
        let found = Theme::from_config_name("dark");

        // assert
        assert_eq!(found, Some(Theme::Dark));
        assert_eq!(Theme::from_config_name("sepia"), None);
    }

    #[test]
    fn the_system_theme_follows_the_platform() {
        // act
        let dark = visuals(Theme::System, true);
        let light = visuals(Theme::System, false);

        // assert
        assert!(dark.dark_mode);
        assert!(!light.dark_mode);
    }

    #[test]
    fn an_explicit_theme_ignores_the_platform() {
        // act
        let dark = visuals(Theme::Dark, false);
        let light = visuals(Theme::Light, true);

        // assert
        assert!(dark.dark_mode);
        assert!(!light.dark_mode);
    }

    #[test]
    fn every_theme_has_a_unique_spelling() {
        // act
        let spellings = [
            Theme::System.as_str(),
            Theme::Light.as_str(),
            Theme::Dark.as_str(),
        ];

        // assert
        let mut sorted = spellings.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), spellings.len());
    }
}
