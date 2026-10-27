//! The translation of the events of the framework into the events of the
//! catalogue.
//!
//! WHAT: the state of the mouse and of the finger ([`PointerTranslator`])
//! and the events of the catalogue that come out of it
//! ([`PointerEvent`](crate::keymap_view::PointerEvent) and
//! [`Gesture`](crate::keymap_view::Gesture)).
//! WHY: the framework of the interface delivers raw events — a button was
//! pressed, a finger moved, the wheel was turned, the fingers came closer.
//! The catalogue of the bindings, on the other hand, names the signals that
//! a person uses: a click, a double click, a tap, a long press, a swipe.
//! The translation is therefore one place, and it lives here so that the
//! recognition of the gestures can be tested on its own.
//!
//! HOW THE RECOGNITION WORKS:
//!
//! ```text
//! the finger goes down ......... the place and the time are remembered
//! the finger rests ............. after LONG_PRESS the long press is sent
//! the finger moves ............. a move over SWIPE_DISTANCE is a swipe
//! the finger lifts ............. a tap, a long press or a swipe goes out
//!
//! the mouse button goes down ... the place and the time are remembered
//! the mouse button lifts ....... a click, a double click or a drag goes out
//! the wheel turns .............. a scroll goes out
//! the fingers come closer ...... a pinch goes out
//! ```
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::input::pointer::PointerTranslator;
//! use phocommander_ui::keymap_view::InputEvent;
//!
//! let mut translator = PointerTranslator::new();
//! let event = egui::Event::PointerButton {
//!     pos: egui::pos2(10.0, 20.0),
//!     button: egui::PointerButton::Primary,
//!     pressed: true,
//!     modifiers: egui::Modifiers::default(),
//! };
//! assert_eq!(translator.translate(&event, 0.0), None);
//! ```

use egui::{Pos2, Vec2};

use crate::keymap_view::{Direction, Gesture, InputEvent, PinchDirection, PointerEvent};

#[cfg(test)]
mod tests;

/// The distance in points that a finger may travel and still count as a tap.
pub const TAP_DISTANCE: f32 = 10.0;

/// The distance in points that a finger has to travel to count as a swipe.
pub const SWIPE_DISTANCE: f32 = 40.0;

/// The time in seconds that a finger has to rest to count as a long press.
pub const LONG_PRESS_SECONDS: f64 = 0.5;

/// The time in seconds in which two clicks count as a double click.
pub const DOUBLE_CLICK_SECONDS: f64 = 0.4;

/// The state of one finger.
#[derive(Debug, Clone, Copy)]
struct Finger {
    /// Where the finger went down.
    start: Pos2,
    /// Where the finger was the last time.
    last: Pos2,
    /// When the finger went down.
    since: f64,
    /// Whether the long press has already gone out.
    long_press_sent: bool,
    /// Whether the swipe has already gone out.
    swipe_sent: bool,
}

/// The state of the mouse.
///
/// WHY: the state exists while the button is down. The time of the press
///       is not needed — the time of the *click* is remembered separately,
///       and it is the click that the double click looks at.
#[derive(Debug, Clone, Copy)]
struct Mouse {
    /// Where the button went down.
    start: Pos2,
}

/// The translator of the events of the framework.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::input::pointer::PointerTranslator;
///
/// let mut translator = PointerTranslator::new();
/// assert!(translator.poll(0.0).is_none());
/// ```
#[derive(Debug, Clone)]
pub struct PointerTranslator {
    /// The finger that is on the surface, if one is.
    finger: Option<Finger>,

    /// The state of the mouse.
    mouse: Option<Mouse>,

    /// The time and the place of the last click, for the double click.
    last_click: Option<(f64, Pos2)>,
}

impl Default for PointerTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl PointerTranslator {
    /// A translator that has not seen anything yet.
    pub fn new() -> Self {
        Self {
            finger: None,
            mouse: None,
            last_click: None,
        }
    }

    /// Translates one event of the framework into an event of the catalogue.
    ///
    /// WHY: the events that carry no meaning of their own — a finger that
    /// goes down, a finger that moves — leave the translator with nothing.
    /// The event that carries the meaning — the tap, the swipe, the click —
    /// comes out of it.
    pub fn translate(&mut self, event: &egui::Event, time: f64) -> Option<InputEvent> {
        match event {
            egui::Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers: _,
            } => self.pointer_button(*pos, *button, *pressed, time),
            egui::Event::Touch { phase, pos, .. } => self.touch(*phase, *pos, time),
            egui::Event::MouseWheel { delta, .. } => self.wheel(*delta),
            egui::Event::Zoom(scale) => self.zoom(*scale),
            _ => None,
        }
    }

    /// The event that the state of the translator produces without a new
    /// event of the framework.
    ///
    /// WHY: the long press is not an event. The finger rests, and nothing
    /// happens until the time of the long press has passed. The frame has
    /// to ask therefore, once per frame, whether the finger has rested long
    /// enough.
    pub fn poll(&mut self, time: f64) -> Option<InputEvent> {
        let finger = self.finger?;
        if finger.long_press_sent {
            return None;
        }
        if time - finger.since < LONG_PRESS_SECONDS {
            return None;
        }

        // WHY: the long press goes out only once per finger, and the finger
        //       stays until it lifts.
        if let Some(finger) = self.finger.as_mut() {
            finger.long_press_sent = true;
        }
        Some(InputEvent::Gesture(Gesture::LongPress))
    }

    /// The handling of a button of the mouse.
    fn pointer_button(
        &mut self,
        pos: Pos2,
        button: egui::PointerButton,
        pressed: bool,
        time: f64,
    ) -> Option<InputEvent> {
        match (button, pressed) {
            (egui::PointerButton::Primary, true) => {
                self.mouse = Some(Mouse { start: pos });
                None
            }
            (egui::PointerButton::Primary, false) => {
                let mouse = self.mouse.take()?;
                let weg = distance(mouse.start, pos);
                if weg >= SWIPE_DISTANCE {
                    return Some(InputEvent::Pointer(PointerEvent::Drag));
                }
                // WHY: two clicks in quick succession at the same place are
                //       a double click. The first click was a click.
                if let Some((zeit, platz)) = self.last_click
                    && time - zeit <= DOUBLE_CLICK_SECONDS
                    && distance(platz, pos) <= TAP_DISTANCE
                {
                    self.last_click = None;
                    return Some(InputEvent::Pointer(PointerEvent::DoubleClick));
                }
                self.last_click = Some((time, pos));
                Some(InputEvent::Pointer(PointerEvent::Click))
            }
            (egui::PointerButton::Secondary, true) => {
                Some(InputEvent::Pointer(PointerEvent::RightClick))
            }
            _ => None,
        }
    }

    /// The handling of a finger on the surface.
    fn touch(&mut self, phase: egui::TouchPhase, pos: Pos2, time: f64) -> Option<InputEvent> {
        match phase {
            egui::TouchPhase::Start => {
                self.finger = Some(Finger {
                    start: pos,
                    last: pos,
                    since: time,
                    long_press_sent: false,
                    swipe_sent: false,
                });
                None
            }
            egui::TouchPhase::Move => {
                let finger = self.finger.as_mut()?;
                finger.last = pos;
                if finger.swipe_sent {
                    return None;
                }
                if distance(finger.start, pos) < SWIPE_DISTANCE {
                    return None;
                }
                finger.swipe_sent = true;
                Some(InputEvent::Gesture(Gesture::Swipe(direction_of(
                    pos - finger.start,
                ))))
            }
            egui::TouchPhase::End => {
                let finger = self.finger.take()?;
                if finger.swipe_sent {
                    // WHY: the swipe went out when the finger passed the
                    //       distance. The finger that lifts afterwards must
                    //       not say it a second time.
                    return None;
                }
                let weg = distance(finger.start, finger.last);
                if weg >= SWIPE_DISTANCE {
                    return Some(InputEvent::Gesture(Gesture::Swipe(direction_of(
                        finger.last - finger.start,
                    ))));
                }
                if finger.long_press_sent {
                    // WHY: the long press has already gone out. A second
                    //       event would say the same thing twice.
                    return None;
                }
                if time - finger.since >= LONG_PRESS_SECONDS {
                    return Some(InputEvent::Gesture(Gesture::LongPress));
                }
                Some(InputEvent::Gesture(Gesture::Tap))
            }
            egui::TouchPhase::Cancel => {
                self.finger = None;
                None
            }
        }
    }

    /// The handling of the scroll wheel.
    fn wheel(&mut self, delta: Vec2) -> Option<InputEvent> {
        if delta.y.abs() < f32::EPSILON {
            return None;
        }
        let schritte = if delta.y > 0.0 { 1 } else { -1 };
        Some(InputEvent::Pointer(PointerEvent::Scroll(schritte)))
    }

    /// The handling of the distance between two fingers.
    fn zoom(&mut self, scale: f32) -> Option<InputEvent> {
        if (scale - 1.0).abs() < 0.01 {
            return None;
        }
        let richtung = if scale > 1.0 {
            PinchDirection::Out
        } else {
            PinchDirection::In
        };
        Some(InputEvent::Gesture(Gesture::Pinch(richtung)))
    }
}

/// The distance between two places.
fn distance(a: Pos2, b: Pos2) -> f32 {
    (b - a).length()
}

/// The direction of a movement.
fn direction_of(bewegung: Vec2) -> Direction {
    // WHY: the direction of the longer side wins. A movement that is
    //       mostly to the side and a little up is a swipe to the side.
    if bewegung.x.abs() >= bewegung.y.abs() {
        if bewegung.x > 0.0 {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if bewegung.y > 0.0 {
        Direction::Down
    } else {
        Direction::Up
    }
}
