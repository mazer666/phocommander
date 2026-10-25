//! The metrics of the platform.
//!
//! WHAT: the platform the program runs on ([`Platform`]) and the metrics
//!       of the work surface on that platform ([`Metrics`]).
//! WHY: the specification demands that the surface does not look like a web
//!       application in a window: the row height, the font size, the focus
//!       frame and the depth of the sidebar are properties of the platform
//!       (section 6.5). A television needs a focus frame that is seen from
//!       three metres and a finger needs a row of forty-four points.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::metrics::{Metrics, Platform};
//!
//! let metrics = Metrics::for_platform(Platform::MacOs);
//! assert!(metrics.row_height() >= 18.0);
//! ```

/// The platform the program runs on.
///
/// WHY: the enumeration mirrors the one of the platform module. The two
/// cannot be the same type, because a crate below `crates/` may not depend
/// on a sibling crate — the shell maps the one into the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Platform {
    /// macOS.
    #[default]
    MacOs,

    /// Windows.
    Windows,

    /// Linux.
    Linux,

    /// FreeBSD, NetBSD or OpenBSD.
    Bsd,

    /// Android, including the television and the headset variants.
    Android,

    /// iPadOS.
    Ios,
}

impl Platform {
    /// Every platform, in a stable order.
    pub const fn all() -> [Platform; 6] {
        [
            Platform::MacOs,
            Platform::Windows,
            Platform::Linux,
            Platform::Bsd,
            Platform::Android,
            Platform::Ios,
        ]
    }

    /// The spelling of the platform in the configuration file.
    pub const fn config_name(&self) -> &'static str {
        match self {
            Platform::MacOs => "macos",
            Platform::Windows => "windows",
            Platform::Linux => "linux",
            Platform::Bsd => "bsd",
            Platform::Android => "android",
            Platform::Ios => "ios",
        }
    }

    /// The platform with the given spelling in the configuration file.
    pub fn from_config_name(name: &str) -> Option<Platform> {
        Platform::all()
            .into_iter()
            .find(|platform| platform.config_name() == name)
    }

    /// Whether the platform is operated with a finger.
    pub const fn is_touch(&self) -> bool {
        matches!(self, Platform::Android | Platform::Ios)
    }
}

/// The metrics of the work surface.
///
/// WHY: the metrics are the difference between a native program and a web
/// form in a window. Every value is a decision of the platform, and the
/// context may change it again — a television needs larger type and a
/// stronger focus frame than a laptop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    row_height: f32,
    font_size: f32,
    focus_frame_width: f32,
    sidebar_depth: f32,
    column_size_width: f32,
    column_date_width: f32,
    column_type_width: f32,
}

impl Metrics {
    /// The metrics of a platform.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::metrics::{Metrics, Platform};
    ///
    /// let desktop = Metrics::for_platform(Platform::Linux);
    /// let touch = Metrics::for_platform(Platform::Android);
    /// assert!(touch.row_height() > desktop.row_height());
    /// ```
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOs => Self {
                row_height: 20.0,
                font_size: 13.0,
                focus_frame_width: 2.0,
                sidebar_depth: 220.0,
                column_size_width: 90.0,
                column_date_width: 130.0,
                column_type_width: 90.0,
            },
            Platform::Windows => Self {
                row_height: 20.0,
                font_size: 13.0,
                focus_frame_width: 1.5,
                sidebar_depth: 220.0,
                column_size_width: 90.0,
                column_date_width: 130.0,
                column_type_width: 90.0,
            },
            Platform::Linux | Platform::Bsd => Self {
                row_height: 20.0,
                font_size: 13.0,
                focus_frame_width: 1.5,
                sidebar_depth: 200.0,
                column_size_width: 90.0,
                column_date_width: 130.0,
                column_type_width: 90.0,
            },
            Platform::Android | Platform::Ios => Self {
                // WHY: a finger needs a target of forty-four points, and the
                // specification names that number for the touch context.
                row_height: 44.0,
                font_size: 16.0,
                focus_frame_width: 3.0,
                sidebar_depth: 300.0,
                column_size_width: 110.0,
                column_date_width: 150.0,
                column_type_width: 110.0,
            },
        }
    }

    /// The metrics for an interaction context.
    ///
    /// WHY: the television is read from about three metres, so the type is
    /// larger and the focus frame is markedly stronger than on the desktop
    /// (section 6.5).
    pub fn for_context(self, context: crate::context::Context) -> Self {
        use crate::context::Context;

        match context {
            Context::Dpad => Self {
                row_height: self.row_height * 1.8,
                font_size: self.font_size * 1.8,
                focus_frame_width: self.focus_frame_width * 2.0,
                sidebar_depth: self.sidebar_depth,
                column_size_width: self.column_size_width * 1.8,
                column_date_width: self.column_date_width * 1.8,
                column_type_width: self.column_type_width * 1.8,
            },
            Context::Touch => Self {
                row_height: self.row_height.max(44.0),
                font_size: self.font_size.max(16.0),
                focus_frame_width: self.focus_frame_width.max(3.0),
                ..self
            },
            Context::Keyboard | Context::Pointer | Context::Controller => self,
        }
    }

    /// The height of a row of the file list.
    pub const fn row_height(&self) -> f32 {
        self.row_height
    }

    /// The size of the text.
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    /// The width of the frame around the focused element.
    pub const fn focus_frame_width(&self) -> f32 {
        self.focus_frame_width
    }

    /// The depth of the sidebar.
    pub const fn sidebar_depth(&self) -> f32 {
        self.sidebar_depth
    }

    /// The width of the column with the sizes.
    pub const fn column_size_width(&self) -> f32 {
        self.column_size_width
    }

    /// The width of the column with the dates.
    pub const fn column_date_width(&self) -> f32 {
        self.column_date_width
    }

    /// The width of the column with the types.
    pub const fn column_type_width(&self) -> f32 {
        self.column_type_width
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::context::Context;

    #[test]
    fn every_platform_has_a_unique_name() {
        // act
        let names: Vec<&str> = Platform::all()
            .iter()
            .map(|platform| platform.config_name())
            .collect();

        // assert
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn the_touch_platforms_have_a_tall_row() {
        // WHY: the specification names forty-four points as the smallest
        // target a finger can hit.
        // act
        let android = Metrics::for_platform(Platform::Android);
        let linux = Metrics::for_platform(Platform::Linux);

        // assert
        assert!(android.row_height() >= 44.0);
        assert!(linux.row_height() < 44.0);
    }

    #[test]
    fn the_television_needs_larger_type_and_a_stronger_frame() {
        // arrange
        let desktop = Metrics::for_platform(Platform::Linux);

        // act
        let television = desktop.for_context(Context::Dpad);

        // assert
        assert!(television.font_size() > desktop.font_size());
        assert!(television.focus_frame_width() > desktop.focus_frame_width());
        assert!(television.row_height() > desktop.row_height());
    }

    #[test]
    fn the_desktop_keeps_its_metrics_in_the_keyboard_context() {
        // arrange
        let desktop = Metrics::for_platform(Platform::Linux);

        // act
        let keyboard = desktop.for_context(Context::Keyboard);

        // assert
        assert_eq!(keyboard, desktop);
    }
}
