//! The buttons of a gamepad and of a controller of the virtual reality.
//!
//! WHAT: the buttons that a gamepad and a VR controller have ([`ControllerButton`]).
//! WHY: the specification names the signals of the controller context in
//! section 6.2: "trigger, grip, thumbstick, A/B/X/Y, D-pad". These signals
//! are no keys of a keyboard, and a binding that can only be a key would
//! therefore not reach the action in the controller context (FR-101).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::keymap_view::ControllerButton;
//!
//! assert_eq!(ControllerButton::parse("Grip"), Some(ControllerButton::Grip));
//! ```

/// A button of a gamepad or of a VR controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ControllerButton {
    /// The trigger of the index finger.
    Trigger,

    /// The button in the hand.
    Grip,

    /// The stick of the thumb.
    Thumbstick,

    /// The button A.
    A,

    /// The button B.
    B,

    /// The button X.
    X,

    /// The button Y.
    Y,

    /// The directional pad, upwards.
    DpadUp,

    /// The directional pad, downwards.
    DpadDown,

    /// The directional pad, to the left.
    DpadLeft,

    /// The directional pad, to the right.
    DpadRight,
}

impl ControllerButton {
    /// The name of the button, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::ControllerButton;
    ///
    /// assert_eq!(ControllerButton::Grip.as_str(), "Grip");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trigger => "Trigger",
            Self::Grip => "Grip",
            Self::Thumbstick => "Thumbstick",
            Self::A => "A",
            Self::B => "B",
            Self::X => "X",
            Self::Y => "Y",
            Self::DpadUp => "DpadUp",
            Self::DpadDown => "DpadDown",
            Self::DpadLeft => "DpadLeft",
            Self::DpadRight => "DpadRight",
        }
    }

    /// The button of the configuration file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::ControllerButton;
    ///
    /// assert_eq!(ControllerButton::parse("Grip"), Some(ControllerButton::Grip));
    /// assert_eq!(ControllerButton::parse("Trigger"), Some(ControllerButton::Trigger));
    /// ```
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "Trigger" => Some(Self::Trigger),
            "Grip" => Some(Self::Grip),
            "Thumbstick" => Some(Self::Thumbstick),
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            "X" => Some(Self::X),
            "Y" => Some(Self::Y),
            "DpadUp" => Some(Self::DpadUp),
            "DpadDown" => Some(Self::DpadDown),
            "DpadLeft" => Some(Self::DpadLeft),
            "DpadRight" => Some(Self::DpadRight),
            _ => None,
        }
    }
}
