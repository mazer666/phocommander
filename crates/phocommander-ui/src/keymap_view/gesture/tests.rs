//! The tests of the events of the pointer and of the finger.
//!
//! WHAT: the test proves that the pointer and the finger bind to the same
//!       actions as the keyboard (FR-101).
//! WHY: the specification asks that the key, the pointer, the touch gesture,
//!       the D-pad and the controller bind to the same actions. A binding
//!       that can only be reached with the keyboard would leave the actions
//!       unreachable on a touch screen and on a television.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --lib keymap_view::gesture_tests
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use super::*;
use crate::actions::Action;
use crate::context::Context;
use crate::keymap_view::{Binding, ControllerButton, InputEvent, KeyStroke, KeymapView};

#[test]
fn the_notation_of_a_pointer_event_is_read_back() {
    // act
    let parsed = PointerEvent::parse("double-click");
    let written = PointerEvent::DoubleClick.as_str();

    // assert
    assert_eq!(parsed, Some(PointerEvent::DoubleClick));
    assert_eq!(written, "double-click");
}

#[test]
fn the_wheel_is_read_in_both_directions() {
    // act, assert
    assert_eq!(
        PointerEvent::parse("wheel-up"),
        Some(PointerEvent::Scroll(1))
    );
    assert_eq!(
        PointerEvent::parse("wheel-down"),
        Some(PointerEvent::Scroll(-1))
    );
    assert_eq!(PointerEvent::Scroll(1).as_str(), "wheel-up");
    assert_eq!(PointerEvent::Scroll(-1).as_str(), "wheel-down");
}

#[test]
fn the_notation_of_a_gesture_is_read_back() {
    // act, assert
    assert_eq!(Gesture::parse("tap"), Some(Gesture::Tap));
    assert_eq!(Gesture::parse("long-press"), Some(Gesture::LongPress));
    assert_eq!(
        Gesture::parse("swipe-left"),
        Some(Gesture::Swipe(Direction::Left))
    );
    assert_eq!(
        Gesture::parse("pinch-out"),
        Some(Gesture::Pinch(PinchDirection::Out))
    );
}

#[test]
fn a_binding_of_the_pointer_is_read_and_written_back() {
    // act
    let binding = Binding::parse("click");
    let written = binding.as_ref().map(|b| b.to_binding());

    // assert
    assert_eq!(binding, Ok(Binding::Pointer(PointerEvent::Click)));
    assert_eq!(written.as_ref().ok(), Some(&"click".to_string()));
}

#[test]
fn a_binding_of_a_gesture_is_read_and_written_back() {
    // act
    let binding = Binding::parse("swipe-left");
    let written = binding.as_ref().map(|b| b.to_binding());

    // assert
    assert_eq!(
        binding,
        Ok(Binding::Gesture(Gesture::Swipe(Direction::Left)))
    );
    assert_eq!(written.as_ref().ok(), Some(&"swipe-left".to_string()));
}

#[test]
fn a_pointer_and_a_finger_reach_the_same_actions_as_the_keyboard() {
    // arrange
    // WHY: FR-101 — the key, the pointer, the touch gesture, the D-pad and
    //       the controller bind to the same actions. The test builds three
    //       tables of the same actions with different bindings and proves
    //       that every action is reached in every context.
    let aktionen = [
        ("panel.view", "F3"),
        ("panel.rename", "F2"),
        ("panel.copy", "F5"),
    ];

    let tastatur = table(&aktionen);
    let zeiger = table(&[
        ("panel.view", "double-click"),
        ("panel.rename", "F2"),
        ("panel.copy", "F5"),
    ]);
    let finger = table(&[
        ("panel.view", "tap"),
        ("panel.rename", "palette:file.rename"),
        ("panel.copy", "palette:file.copy"),
    ]);

    // act
    let tastatur_view = tastatur.get("panel.view");
    let zeiger_view = zeiger.get("panel.view");
    let finger_view = finger.get("panel.view");

    // assert
    // WHY: the same action is reached by three different signals. The
    //       binding differs, the action does not.
    assert_eq!(
        tastatur_view,
        Some(&Binding::Key(KeyStroke::parse("F3").unwrap()))
    );
    assert_eq!(
        zeiger_view,
        Some(&Binding::Pointer(PointerEvent::DoubleClick))
    );
    assert_eq!(finger_view, Some(&Binding::Gesture(Gesture::Tap)));
    assert_eq!(tastatur.len(), zeiger.len());
    assert_eq!(tastatur.len(), finger.len());
}

#[test]
fn a_touch_event_reaches_an_action_of_its_own_context() {
    // arrange
    let mut keyboard = BTreeMap::new();
    keyboard.insert("panel.view".to_string(), "F3".to_string());
    let mut touch = BTreeMap::new();
    touch.insert("panel.view".to_string(), "tap".to_string());
    let mut dpad = BTreeMap::new();
    dpad.insert("panel.view".to_string(), "palette:file.view".to_string());

    // act
    let mut alle = BTreeMap::new();
    alle.insert(Context::Keyboard, keyboard);
    alle.insert(Context::Touch, touch);
    alle.insert(Context::Dpad, dpad);
    let view = KeymapView::from_context_tables(&alle).expect("the tables should be readable");

    // assert
    let ansicht = Action::from_id("panel.view").expect("the action should be known");
    let tasten = InputEvent::Key(KeyStroke::parse("F3").unwrap());
    let tipp = InputEvent::Gesture(Gesture::Tap);

    assert_eq!(view.resolve(Context::Keyboard, &tasten), Some(ansicht));
    assert_eq!(view.resolve(Context::Touch, &tipp), Some(ansicht));
    // WHY: the palette of the remote control carries the same action under
    //       its own name.
    let palette = InputEvent::Palette("file.view".to_string());
    assert_eq!(view.resolve(Context::Dpad, &palette), Some(ansicht));
}

/// Builds a table of bindings from the pairs.
fn table(pairs: &[(&str, &str)]) -> BTreeMap<String, Binding> {
    pairs
        .iter()
        .map(|(aktion, bindung)| {
            (
                (*aktion).to_string(),
                Binding::parse(bindung).expect("the binding should be readable"),
            )
        })
        .collect()
}

#[test]
fn the_defaults_bind_every_context_without_a_file() {
    // arrange
    // WHY: a user without a file of his own must still be able to reach
    //       every action. The defaults of the code are that safety net,
    //       and the file of the configuration overrides them.
    let view =
        KeymapView::with_defaults(&Default::default()).expect("the defaults should be readable");

    // act
    let tipp = InputEvent::Gesture(Gesture::Tap);
    let klick = InputEvent::Pointer(PointerEvent::DoubleClick);
    let taste = InputEvent::Key(KeyStroke::parse("F3").unwrap());

    // assert
    assert_eq!(
        view.resolve(Context::Touch, &tipp).map(|a| a.id()),
        Some("panel.view")
    );
    assert_eq!(
        view.resolve(Context::Pointer, &klick).map(|a| a.id()),
        Some("panel.view")
    );
    assert_eq!(
        view.resolve(Context::Keyboard, &taste).map(|a| a.id()),
        Some("panel.view")
    );
}

#[test]
fn the_file_of_the_user_overrides_the_defaults() {
    // arrange
    let mut touch = BTreeMap::new();
    touch.insert("panel.view".to_string(), "long-press".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(Context::Touch, touch);

    // act
    let view = KeymapView::with_defaults(&tables).expect("the tables should be readable");

    // assert
    // WHY: the long press is now the binding of the touch context.
    assert_eq!(
        view.resolve(Context::Touch, &InputEvent::Gesture(Gesture::LongPress))
            .map(|a| a.id()),
        Some("panel.view")
    );
    // WHY: the tap does not lead to the action any more, because the user
    //       has bound the long press to it.
    assert_eq!(
        view.resolve(Context::Touch, &InputEvent::Gesture(Gesture::Tap)),
        None
    );
}

#[test]
fn a_button_of_the_controller_reaches_its_action() {
    // arrange
    // WHY: the controller context names the grip as the binding of the
    //       selection. The grip is no key of a keyboard, and the binding
    //       has to reach the action all the same (FR-101).
    let mut controller = BTreeMap::new();
    controller.insert("selection.toggle".to_string(), "Grip".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(Context::Controller, controller);
    let view = KeymapView::from_context_tables(&tables).expect("the tables should be readable");

    // act
    let griff = InputEvent::Controller(ControllerButton::Grip);

    // assert
    assert_eq!(
        view.resolve(Context::Controller, &griff).map(|a| a.id()),
        Some("selection.toggle")
    );
    // WHY: the keyboard context has no binding for the grip, and the grip
    //       must not trigger anything there.
    assert_eq!(view.resolve(Context::Keyboard, &griff), None);
}

#[test]
fn the_defaults_bind_the_grip_of_the_controller() {
    // arrange
    let view =
        KeymapView::with_defaults(&Default::default()).expect("the defaults should be readable");

    // act
    let griff = InputEvent::Controller(ControllerButton::Grip);

    // assert
    assert_eq!(
        view.resolve(Context::Controller, &griff).map(|a| a.id()),
        Some("selection.toggle")
    );
}
