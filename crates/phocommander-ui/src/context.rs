//! The interaction contexts of the user interface.
//!
//! WHAT: the context in which the user operates the program ([{Context}]).
//! WHY: the same action needs a different binding, a different layout and a
//!       different metric in every context: the keyboard of a laptop, the
//!       finger on a tablet, the D-pad of a television remote control. The
//!       context is therefore part of the state of the interface and not a
//!       detail of one widget.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::Context;
//!
//! assert_eq!(Context::Dpad.config_name(), "dpad");
//! assert!(Context::all().contains(&Context::Keyboard));
//! ```

/// The context in which the user operates the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Context {
/// A physical keyboard.
    #[default]
    Keyboard,

/// A mouse or a trackpad.
    Pointer,

/// A finger on a touch screen.
    Touch,

/// The directional pad of a television remote control.
    Dpad,

/// A gamepad or a VR controller.
    Controller,
}

impl Context {
/// Every context, in a stable order.
///
/// WHY: the order is fixed so that a list of contexts — in the
/// configuration file, in the documentation or in a test — always reads
/// the same way.
    pub const fn all() -> [Context; 5] {
        [
            Context::Keyboard,
            Context::Pointer,
            Context::Touch,
            Context::Dpad,
            Context::Controller,
        ]
    }

/// The spelling of the context in the configuration file.
    pub const fn config_name(&self) -> &'static str {
        match self {
            Context::Keyboard => "keyboard",
            Context::Pointer => "pointer",
            Context::Touch => "touch",
            Context::Dpad => "dpad",
            Context::Controller => "controller",
        }
    }

/// The context with the given spelling in the configuration file.
    pub fn from_config_name(name: &str) -> Option<Self> {
        Context::all()
            .into_iter()
            .find(|context| context.config_name() == name)
    }

/// Whether the context shows one panel at a time.
///
/// WHY: two panels side by side are unreadable on a television, and a
/// finger on a tablet reaches one list at a time (section 6.5 of the
/// specification). The layout of the work surface follows this answer.
    pub const fn shows_one_panel(&self) -> bool {
        matches!(self, Context::Touch | Context::Dpad)
    }

/// Whether the context has a pointer.
///
/// WHY: without a pointer there is no hover, and the focus therefore has
/// to be a visible, permanent state (section 6.4).
    pub const fn has_pointer(&self) -> bool {
        matches!(self, Context::Pointer | Context::Touch)
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.config_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_context_has_a_unique_configuration_name() {
        // act
        let names: Vec<&str> = Context::all()
            .iter()
            .map(|context| context.config_name())
            .collect();

        // assert
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn finds_a_context_by_its_configuration_name() {
        // act
        let found = Context::from_config_name("dpad");

        // assert
        assert_eq!(found, Some(Context::Dpad));
        assert_eq!(Context::from_config_name("gamepad"), None);
    }

    #[test]
    fn the_television_and_the_tablet_show_one_panel() {
        // act
        let dpad = Context::Dpad.shows_one_panel();
        let touch = Context::Touch.shows_one_panel();
        let keyboard = Context::Keyboard.shows_one_panel();

        // assert
        assert!(dpad);
        assert!(touch);
        assert!(!keyboard);
    }

    #[test]
    fn only_the_pointer_context_has_a_pointer() {
        // act
        let pointer = Context::Pointer.has_pointer();
        let touch = Context::Touch.has_pointer();
        let dpad = Context::Dpad.has_pointer();

        // assert
        assert!(pointer);
        assert!(touch);
        assert!(!dpad);
    }
}
