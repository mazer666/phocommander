//! The events of the pointer and of the finger.
//!
//! WHAT: the event of a mouse or a trackpad ([`PointerEvent`]), the gesture
//! of a finger ([`Gesture`]) and the direction of a swipe ([`Direction`]).
//! WHY: the specification asks that the key, the pointer, the touch gesture,
//! the D-pad and the controller bind to the **same actions** (FR-101).
//! A binding that names only a key can therefore not reach an action that
//! is bound to a tap or to a click. The two types below give the other two
//! devices their signals.
//! The signals are the ones the specification names for the two contexts:
//! the pointer knows the pointer itself, the click, the double click, the
//! right click, the drag and the scroll wheel; the finger knows the tap,
//! the long press, the swipe, the scroll with two fingers and the pinch.
//! BOUNDARY: this module names the signals. It does not read them — the
//! input layer of the platform delivers them — and it knows nothing of the
//! surface and of the actions.

#[cfg(test)]
mod tests;

/// The event of a mouse or a trackpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerEvent {
    /// The primary button was pressed and released.
    Click,

    /// The primary button was pressed twice in quick succession.
    DoubleClick,

    /// The secondary button was pressed.
    RightClick,

    /// The pointer was dragged.
    Drag,

    /// The scroll wheel was turned. A positive number counts the notches
    /// away from the user, a negative number the notches towards the user.
    Scroll(i32),
}

impl PointerEvent {
    /// The name of the event, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::PointerEvent;
    ///
    /// assert_eq!(PointerEvent::Click.as_str(), "click");
    /// assert_eq!(PointerEvent::Scroll(1).as_str(), "wheel-up");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::DoubleClick => "double-click",
            Self::RightClick => "right-click",
            Self::Drag => "drag",
            Self::Scroll(steps) if *steps > 0 => "wheel-up",
            Self::Scroll(steps) if *steps < 0 => "wheel-down",
            Self::Scroll(_) => "wheel",
        }
    }

    /// The event of the configuration file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::PointerEvent;
    ///
    /// assert_eq!(PointerEvent::parse("click"), Some(PointerEvent::Click));
    /// assert_eq!(PointerEvent::parse("wheel-up"), Some(PointerEvent::Scroll(1)));
    /// ```
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "click" => Some(Self::Click),
            "double-click" => Some(Self::DoubleClick),
            "right-click" => Some(Self::RightClick),
            "drag" => Some(Self::Drag),
            "wheel-up" => Some(Self::Scroll(1)),
            "wheel-down" => Some(Self::Scroll(-1)),
            "wheel" => Some(Self::Scroll(0)),
            _ => None,
        }
    }
}

/// The direction of a swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// To the left.
    Left,

    /// To the right.
    Right,

    /// Upwards.
    Up,

    /// Downwards.
    Down,
}

impl Direction {
    /// The name of the direction, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::Direction;
    ///
    /// assert_eq!(Direction::Left.as_str(), "left");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

/// The gesture of a finger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gesture {
    /// The finger touched the screen and was lifted again.
    Tap,

    /// The finger rested on the screen for a long time.
    LongPress,

    /// The finger was dragged across the screen.
    Swipe(Direction),

    /// Two fingers scrolled the list.
    TwoFingerScroll(i32),

    /// Two fingers changed the distance between them.
    Pinch(PinchDirection),
}

/// The direction of a pinch.
///
/// WHY: the pinch carries a direction and not a number. A number of the
///       distance between the fingers is a value of `f32`, and a value of
///       `f32` can neither be compared for equality nor be put in order,
///       and the table of the bindings needs both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PinchDirection {
    /// The fingers move apart.
    Out,

    /// The fingers move towards each other.
    In,
}

impl PinchDirection {
    /// The name of the direction, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::PinchDirection;
    ///
    /// assert_eq!(PinchDirection::Out.as_str(), "out");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::In => "in",
        }
    }
}

impl Gesture {
    /// The name of the gesture, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::{Direction, Gesture};
    ///
    /// assert_eq!(Gesture::Tap.as_str(), "tap");
    /// assert_eq!(Gesture::Swipe(Direction::Left).as_str(), "swipe-left");
    /// ```
    pub fn as_str(&self) -> String {
        match self {
            Self::Tap => "tap".to_string(),
            Self::LongPress => "long-press".to_string(),
            Self::Swipe(direction) => format!("swipe-{}", direction.as_str()),
            Self::TwoFingerScroll(steps) if *steps > 0 => "two-finger-up".to_string(),
            Self::TwoFingerScroll(steps) if *steps < 0 => "two-finger-down".to_string(),
            Self::TwoFingerScroll(_) => "two-finger-scroll".to_string(),
            Self::Pinch(direction) => format!("pinch-{}", direction.as_str()),
        }
    }

    /// The gesture of the configuration file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::{Direction, Gesture};
    ///
    /// assert_eq!(Gesture::parse("tap"), Some(Gesture::Tap));
    /// assert_eq!(Gesture::parse("swipe-left"), Some(Gesture::Swipe(Direction::Left)));
    /// ```
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "tap" => Some(Self::Tap),
            "long-press" => Some(Self::LongPress),
            "swipe-left" => Some(Self::Swipe(Direction::Left)),
            "swipe-right" => Some(Self::Swipe(Direction::Right)),
            "swipe-up" => Some(Self::Swipe(Direction::Up)),
            "swipe-down" => Some(Self::Swipe(Direction::Down)),
            "two-finger-up" => Some(Self::TwoFingerScroll(1)),
            "two-finger-down" => Some(Self::TwoFingerScroll(-1)),
            "two-finger-scroll" => Some(Self::TwoFingerScroll(0)),
            "pinch-out" => Some(Self::Pinch(PinchDirection::Out)),
            "pinch-in" => Some(Self::Pinch(PinchDirection::In)),
            _ => None,
        }
    }
}
