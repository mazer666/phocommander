//! The theme of Phocommander.
//!
//! WHAT: the colour scheme the user chose ([`Theme`]), the colours of that
//!       scheme ([`Palette`]) and the egui visuals that belong to it.
//! WHY: the theme is a setting of the user, but the colours that egui draws
//!       with are not. The translation from one to the other happens here, in
//!       the only crate that knows about egui, so that the rest of the
//!       application can talk about "dark" instead of about colour values.
//!       The colours are chosen so that the text reaches a contrast of at
//!       least 4.5:1 and the focus frame at least 3:1 (NFR-11).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::{visuals, Theme};
//!
//! assert!(visuals(Theme::Dark, false).dark_mode);
//! assert!(!visuals(Theme::Light, true).dark_mode);
//! ```

use egui::Color32;

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

/// The colours of the work surface.
///
/// WHY: the colours are the difference between a program that belongs to
/// the system and a program that looks like a web page. The values are
/// chosen so that the text reaches a contrast of at least 4.5:1 against the
/// background of the panel and the focus frame at least 3:1 (NFR-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// The colour behind the whole window.
    pub background: Color32,

    /// The colour of a panel.
    pub panel: Color32,

    /// The colour of the text of an entry.
    pub text: Color32,

    /// The colour of a text that is less important.
    pub text_dimmed: Color32,

    /// The colour of the frame around the focused element.
    pub focus_frame: Color32,

    /// The colour of the background of a marked entry.
    pub marked: Color32,

    /// The colour of the background of the entry under the cursor.
    pub cursor: Color32,
}

impl Palette {
    /// The colours of the light scheme.
    pub const fn light() -> Self {
        Self {
            background: Color32::from_rgb(0xf2, 0xf2, 0xf2),
            panel: Color32::from_rgb(0xff, 0xff, 0xff),
            text: Color32::from_rgb(0x1a, 0x1a, 0x1a),
            text_dimmed: Color32::from_rgb(0x55, 0x55, 0x55),
            focus_frame: Color32::from_rgb(0x0b, 0x57, 0xd0),
            marked: Color32::from_rgb(0xd6, 0xe4, 0xff),
            cursor: Color32::from_rgb(0xff, 0xf3, 0xc4),
        }
    }

    /// The colours of the dark scheme.
    pub const fn dark() -> Self {
        Self {
            background: Color32::from_rgb(0x1b, 0x1b, 0x1b),
            panel: Color32::from_rgb(0x24, 0x24, 0x24),
            text: Color32::from_rgb(0xe8, 0xe8, 0xe8),
            text_dimmed: Color32::from_rgb(0xa4, 0xa4, 0xa4),
            focus_frame: Color32::from_rgb(0x8a, 0xb4, 0xf8),
            marked: Color32::from_rgb(0x2f, 0x4a, 0x7a),
            cursor: Color32::from_rgb(0x3b, 0x3b, 0x2a),
        }
    }
}

/// The contrast ratio of two colours, as the WCAG 2 defines it.
///
/// WHY: the specification demands a text contrast of at least 4.5:1 and a
/// focus frame of at least 3:1 (NFR-11). A ratio that is computed can be
/// checked; a ratio that is estimated is a wish.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::theme::contrast_ratio;
/// use egui::Color32;
///
/// let black = Color32::from_rgb(0, 0, 0);
/// let white = Color32::from_rgb(255, 255, 255);
/// assert!(contrast_ratio(black, white) > 20.0);
/// ```
pub fn contrast_ratio(foreground: Color32, background: Color32) -> f32 {
    let first = relative_luminance(foreground);
    let second = relative_luminance(background);
    let lighter = first.max(second);
    let darker = first.min(second);
    (lighter + 0.05) / (darker + 0.05)
}

/// The relative luminance of a colour, as the WCAG 2 defines it.
fn relative_luminance(color: Color32) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = value as f32 / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
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

    #[test]
    fn the_text_of_the_panels_reaches_the_contrast_the_specification_names() {
        // WHY: NFR-11 — the text needs a contrast of at least 4.5:1.
        for palette in [Palette::light(), Palette::dark()] {
            assert!(
                contrast_ratio(palette.text, palette.panel) >= 4.5,
                "the text of the panel has a contrast of {}",
                contrast_ratio(palette.text, palette.panel)
            );
            assert!(
                contrast_ratio(palette.text_dimmed, palette.panel) >= 4.5,
                "the dimmed text of the panel has a contrast of {}",
                contrast_ratio(palette.text_dimmed, palette.panel)
            );
        }
    }

    #[test]
    fn the_focus_frame_reaches_the_contrast_the_specification_names() {
        // WHY: NFR-11 — the focus frame needs a contrast of at least 3:1.
        for palette in [Palette::light(), Palette::dark()] {
            assert!(
                contrast_ratio(palette.focus_frame, palette.panel) >= 3.0,
                "the focus frame has a contrast of {}",
                contrast_ratio(palette.focus_frame, palette.panel)
            );
        }
    }

    #[test]
    fn the_contrast_of_a_colour_with_itself_is_one() {
        // act
        let ratio = contrast_ratio(Color32::from_rgb(20, 40, 60), Color32::from_rgb(20, 40, 60));

        // assert
        assert!((ratio - 1.0).abs() < 0.0001);
    }
}
