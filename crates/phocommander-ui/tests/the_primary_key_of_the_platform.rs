//! The primary key of the platform triggers the actions.
//!
//! WHAT: the test presses the filter key with the control key and with the
//!       command key and reads the filter field of the panel.
//! WHY: the actions are the same on every platform, only the key that
//!       triggers them differs. `Cmd` is the primary key on macOS and on
//!       iPadOS, `Ctrl` everywhere else. The shell was built with the
//!       control key as the default and nobody set the other one — the
//!       user of a mac pressed `Ctrl+F`, the filter did not open, and the
//!       key looked broken. The shell takes the key of the platform
//!       (FR-101).
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_primary_key_of_the_platform
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{app_with, harness};
use phocommander_ui::app::WorkSurfaceApp;
use phocommander_ui::input::PrimaryModifier;
use phocommander_ui::listing::{Entry, EntryKind};

/// The modifiers of a key, with the given primary key.
///
/// WHY: the framework delivers the command key of macOS twice — once as
///       `mac_cmd` and once as the logical `command`. The shell reads the
///       logical one, and the test delivers it the way macOS does.
fn modifiers(primary: PrimaryModifier) -> egui::Modifiers {
    egui::Modifiers {
        ctrl: primary == PrimaryModifier::Control,
        mac_cmd: primary == PrimaryModifier::Command,
        command: primary == PrimaryModifier::Command,
        ..egui::Modifiers::default()
    }
}

/// The shell that listens to the given primary key.
fn app_with_modifier(primary: PrimaryModifier) -> WorkSurfaceApp {
    let app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    app.with_primary_modifier(primary)
}

#[test]
fn the_control_key_opens_the_filter_on_every_platform() {
    // arrange — the shell with the control key as the primary key.
    let app = app_with_modifier(PrimaryModifier::Control);
    let mut harness = harness(app);

    // act — the user presses Ctrl+F.
    harness.key_press_modifiers(modifiers(PrimaryModifier::Control), egui::Key::F);
    harness.step();

    // assert — the filter field stands open.
    let panel = harness.state().surface().active();
    assert!(
        panel.is_filter_editing(),
        "the control key should open the filter"
    );
}

#[test]
fn the_command_key_opens_the_filter_on_mac() {
    // arrange — the shell with the command key as the primary key, as the
    // composition root builds it on macOS.
    let app = app_with_modifier(PrimaryModifier::Command);
    let mut harness = harness(app);

    // act — the user presses Cmd+F.
    harness.key_press_modifiers(modifiers(PrimaryModifier::Command), egui::Key::F);
    harness.step();

    // assert — the filter field stands open.
    let panel = harness.state().surface().active();
    assert!(
        panel.is_filter_editing(),
        "the command key should open the filter"
    );
}
