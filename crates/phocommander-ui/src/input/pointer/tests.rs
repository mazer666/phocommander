//! The tests of the translation of the events of the framework.
//!
//! WHAT: the test proves that the raw events of the framework become the
//!       signals of the catalogue — the click, the double click, the right
//!       click, the drag, the scroll, the tap, the long press, the swipe and
//!       the pinch.
//! WHY: the recognition of the gestures is the part of FR-101 that the
//!       user feels. A table that binds a tap but a surface that never
//!       recognises a tap is a promise without a feature.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --lib input::pointer::tests
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::keymap_view::{Gesture, PointerEvent};

/// A press of the primary button of the mouse.
fn druecken(position: egui::Pos2) -> egui::Event {
    egui::Event::PointerButton {
        pos: position,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    }
}

/// A release of the primary button of the mouse.
fn loslassen(position: egui::Pos2) -> egui::Event {
    egui::Event::PointerButton {
        pos: position,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    }
}

/// A finger on the surface.
fn finger(phase: egui::TouchPhase, position: egui::Pos2) -> egui::Event {
    egui::Event::Touch {
        device_id: egui::TouchDeviceId(1),
        id: egui::TouchId(1),
        phase,
        pos: position,
        force: None,
    }
}

#[test]
fn a_press_and_a_release_of_the_mouse_make_a_click() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    let gedrueckt = translator.translate(&druecken(egui::pos2(10.0, 10.0)), 0.0);
    let geklickt = translator.translate(&loslassen(egui::pos2(11.0, 11.0)), 0.1);

    // assert
    assert_eq!(gedrueckt, None);
    assert_eq!(geklickt, Some(InputEvent::Pointer(PointerEvent::Click)));
}

#[test]
fn two_clicks_in_quick_succession_make_a_double_click() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    translator.translate(&druecken(egui::pos2(10.0, 10.0)), 0.0);
    let erster = translator.translate(&loslassen(egui::pos2(10.0, 10.0)), 0.1);
    translator.translate(&druecken(egui::pos2(10.0, 10.0)), 0.2);
    let zweiter = translator.translate(&loslassen(egui::pos2(10.0, 10.0)), 0.3);

    // assert
    assert_eq!(erster, Some(InputEvent::Pointer(PointerEvent::Click)));
    assert_eq!(
        zweiter,
        Some(InputEvent::Pointer(PointerEvent::DoubleClick))
    );
}

#[test]
fn two_clicks_that_are_far_apart_stay_two_clicks() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    translator.translate(&druecken(egui::pos2(10.0, 10.0)), 0.0);
    translator.translate(&loslassen(egui::pos2(10.0, 10.0)), 0.1);
    translator.translate(&druecken(egui::pos2(200.0, 200.0)), 0.2);
    let zweiter = translator.translate(&loslassen(egui::pos2(200.0, 200.0)), 0.3);

    // assert
    assert_eq!(zweiter, Some(InputEvent::Pointer(PointerEvent::Click)));
}

#[test]
fn the_right_button_of_the_mouse_makes_a_right_click() {
    // arrange
    let mut translator = PointerTranslator::new();
    let event = egui::Event::PointerButton {
        pos: egui::pos2(10.0, 10.0),
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    };

    // act
    let ergebnis = translator.translate(&event, 0.0);

    // assert
    assert_eq!(
        ergebnis,
        Some(InputEvent::Pointer(PointerEvent::RightClick))
    );
}

#[test]
fn a_mouse_that_travels_far_makes_a_drag() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    translator.translate(&druecken(egui::pos2(10.0, 10.0)), 0.0);
    let gezogen = translator.translate(&loslassen(egui::pos2(120.0, 10.0)), 0.2);

    // assert
    assert_eq!(gezogen, Some(InputEvent::Pointer(PointerEvent::Drag)));
}

#[test]
fn the_wheel_makes_a_scroll_in_both_directions() {
    // arrange
    let mut translator = PointerTranslator::new();
    let nach_unten = egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, 12.0),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: egui::Modifiers::default(),
    };
    let nach_oben = egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -12.0),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: egui::Modifiers::default(),
    };

    // act
    let runter = translator.translate(&nach_unten, 0.0);
    let rauf = translator.translate(&nach_oben, 0.0);

    // assert
    assert_eq!(runter, Some(InputEvent::Pointer(PointerEvent::Scroll(1))));
    assert_eq!(rauf, Some(InputEvent::Pointer(PointerEvent::Scroll(-1))));
}

#[test]
fn a_finger_that_goes_and_comes_makes_a_tap() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    let hinunter = translator.translate(
        &finger(egui::TouchPhase::Start, egui::pos2(50.0, 50.0)),
        0.0,
    );
    let herunter =
        translator.translate(&finger(egui::TouchPhase::End, egui::pos2(52.0, 51.0)), 0.2);

    // assert
    assert_eq!(hinunter, None);
    assert_eq!(herunter, Some(InputEvent::Gesture(Gesture::Tap)));
}

#[test]
fn a_finger_that_rests_makes_a_long_press() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    translator.translate(
        &finger(egui::TouchPhase::Start, egui::pos2(50.0, 50.0)),
        0.0,
    );
    let pollt = translator.poll(LONG_PRESS_SECONDS + 0.1);
    let herunter =
        translator.translate(&finger(egui::TouchPhase::End, egui::pos2(50.0, 50.0)), 1.0);

    // assert
    assert_eq!(pollt, Some(InputEvent::Gesture(Gesture::LongPress)));
    // WHY: the long press goes out only once. The finger that lifts after
    //       it does not say it a second time.
    assert_eq!(herunter, None);
}

#[test]
fn a_finger_that_travels_makes_a_swipe() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    translator.translate(
        &finger(egui::TouchPhase::Start, egui::pos2(200.0, 100.0)),
        0.0,
    );
    let bewegt = translator.translate(
        &finger(egui::TouchPhase::Move, egui::pos2(50.0, 105.0)),
        0.2,
    );
    let herunter =
        translator.translate(&finger(egui::TouchPhase::End, egui::pos2(50.0, 105.0)), 0.3);

    // assert
    assert_eq!(
        bewegt,
        Some(InputEvent::Gesture(Gesture::Swipe(Direction::Left)))
    );
    // WHY: the swipe goes out once, when the finger passes the distance.
    //       The finger that lifts afterwards does not repeat it.
    assert_eq!(herunter, None);
}

#[test]
fn a_finger_that_travels_upward_makes_a_swipe_upward() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    translator.translate(
        &finger(egui::TouchPhase::Start, egui::pos2(100.0, 300.0)),
        0.0,
    );
    let bewegt = translator.translate(
        &finger(egui::TouchPhase::Move, egui::pos2(104.0, 50.0)),
        0.2,
    );

    // assert
    assert_eq!(
        bewegt,
        Some(InputEvent::Gesture(Gesture::Swipe(Direction::Up)))
    );
}

#[test]
fn two_fingers_that_come_closer_make_a_pinch() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    let auseinander = translator.translate(&egui::Event::Zoom(1.2), 0.0);
    let zusammen = translator.translate(&egui::Event::Zoom(0.8), 0.0);
    let unveraendert = translator.translate(&egui::Event::Zoom(1.0), 0.0);

    // assert
    assert_eq!(
        auseinander,
        Some(InputEvent::Gesture(Gesture::Pinch(PinchDirection::Out)))
    );
    assert_eq!(
        zusammen,
        Some(InputEvent::Gesture(Gesture::Pinch(PinchDirection::In)))
    );
    assert_eq!(unveraendert, None);
}

#[test]
fn a_finger_that_is_cancelled_leaves_nothing_behind() {
    // arrange
    let mut translator = PointerTranslator::new();

    // act
    translator.translate(
        &finger(egui::TouchPhase::Start, egui::pos2(50.0, 50.0)),
        0.0,
    );
    translator.translate(
        &finger(egui::TouchPhase::Cancel, egui::pos2(50.0, 50.0)),
        0.1,
    );
    let pollt = translator.poll(2.0);

    // assert
    assert_eq!(pollt, None);
}
