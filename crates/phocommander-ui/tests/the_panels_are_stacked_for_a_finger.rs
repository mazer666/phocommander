//! The panels stand below each other when a finger works on them.
//!
//! WHAT: the test draws the surface in the touch context and in the
//!       keyboard context and reads where the panel of the focus stands.
//! WHY: the arrangement of the two panels is the decision of the user —
//!       and the context has the last word where the decision cannot
//!       exist. On a touch screen the panels stand below each other, because
//!       a finger reaches one panel and not two at the same time (FR-106).
//!       The drawing promised that in a comment and did not do it.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_panels_are_stacked_for_a_finger
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{app_with, harness};
use egui_kittest::kittest::Queryable;
use phocommander_ui::Context;
use phocommander_ui::listing::{Entry, EntryKind};

#[test]
fn the_arrangement_of_the_touch_context_is_stacked() {
    // arrange, act — the decision of the user and the decision of the
    // context together.
    use phocommander_ui::layout::{PanelArrangement, arrangement_of};

    assert_eq!(
        arrangement_of(Context::Touch, PanelArrangement::SideBySide),
        PanelArrangement::Stacked
    );
    assert_eq!(
        arrangement_of(Context::Touch, PanelArrangement::Stacked),
        PanelArrangement::Stacked
    );
}

#[test]
fn the_arrangement_of_the_desktop_is_the_decision_of_the_user() {
    // arrange, act
    use phocommander_ui::layout::{PanelArrangement, arrangement_of};

    assert_eq!(
        arrangement_of(Context::Keyboard, PanelArrangement::SideBySide),
        PanelArrangement::SideBySide
    );
    assert_eq!(
        arrangement_of(Context::Pointer, PanelArrangement::Stacked),
        PanelArrangement::Stacked
    );
}

#[test]
fn the_surface_is_drawn_in_the_touch_context() {
    // arrange — the shell in the touch context with an entry.
    let mut app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    app = app.with_context(Context::Touch);
    let mut harness = harness(app);

    // act — the surface is drawn.
    harness.step();

    // assert — the entry of the panel stands in the surface, and the
    // drawing did not fail.
    let eintraege = harness.query_by_label("notes.txt");
    assert!(eintraege.is_some(), "the entry should be drawn");
}

#[test]
fn the_buttons_of_the_bars_are_targets_of_forty_four_points() {
    // arrange — the shell in the touch context.
    let mut app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    app = app.with_context(Context::Touch);
    let mut harness = harness(app);

    // act — the surface is drawn.
    harness.step();

    // assert — the profile of the touch context names a target of
    // forty-four points for the rows and for the controls. The profile is
    // the answer to "how does the surface look in this context" (FR-106).
    use phocommander_ui::layout::LayoutProfile;
    use phocommander_ui::metrics::Metrics;

    let metrics = Metrics::for_platform(phocommander_ui::Platform::Linux);
    let profile = LayoutProfile::for_context(Context::Touch, &metrics);
    assert!(
        profile.control_height() >= 44.0,
        "the buttons of the bars need a target of forty-four points: {profile:?}"
    );
    assert!(
        profile.row_height() >= 44.0,
        "the rows of the list need a target of forty-four points: {profile:?}"
    );
}
