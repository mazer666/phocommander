//! The state of the session comes back when the user asked for it.
//!
//! WHAT: the test writes a state of the session, builds the application out
//!       of a configuration that asks for it, and reads the directories that
//!       the panels show.
//! WHY: the user writes in the configuration file whether the state of the
//!       last session comes back. The setting was read and never used, and
//!       the program always opened in the working directory — the user
//!       walked to the same place again and again (FR-16).
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-desktop --test the_session_comes_back
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_config::{Config, Security, SessionState, write_session_state};
use phocommander_ui::panel::PanelSide;
use std::path::PathBuf;

#[test]
fn the_session_comes_back_when_the_user_asked_for_it() {
    // arrange — a state of the session in a directory of its own.
    let directory = std::env::temp_dir().join(format!(
        "phocommander-session-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    let state = SessionState::new("/photos", "/documents");
    // WHY: the rule of the security section decides whether the state is
    //       written at all. The profile of the user allows it.
    let sicherheit = Security {
        remember_session: true,
        ..Security::default()
    };
    let regeln = phocommander_config::resolve_security(&sicherheit);
    let geschrieben = write_session_state(&directory, &state, &regeln);
    let geschrieben =
        geschrieben.unwrap_or_else(|error| panic!("the session should be written: {error:?}"));
    assert!(
        geschrieben.is_some(),
        "the session should have been written"
    );

    // act — the state is read and the directories are handed over.
    let gelesen = phocommander_config::load_session_state(&directory);
    let gelesen = gelesen.unwrap_or_else(|error| panic!("the session should be read: {error:?}"));
    let gelesen = gelesen.expect("the session should be there");

    // assert — the two directories of the last time are the ones that the
    // panels show.
    assert_eq!(gelesen.left, PathBuf::from("/photos"));
    assert_eq!(gelesen.right, PathBuf::from("/documents"));
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn the_state_of_the_session_is_not_written_without_the_rule() {
    // arrange — the rule of the safe profile says that the program leaves
    // nothing behind.
    let directory = std::env::temp_dir().join(format!(
        "phocommander-session-sicher-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    let state = SessionState::new("/photos", "/documents");
    let regeln = phocommander_config::EffectiveSecurity::safe();

    // act
    let geschrieben = write_session_state(&directory, &state, &regeln);

    // assert — nothing was written, and the answer says so.
    let geschrieben =
        geschrieben.unwrap_or_else(|error| panic!("the answer should be ok: {error:?}"));
    assert!(
        geschrieben.is_none(),
        "the safe profile writes nothing: {geschrieben:?}"
    );
    assert!(!directory.join("session.toml").exists());
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn the_panels_show_the_directories_of_the_session() {
    // arrange — a shell that was built out of a state of the session.
    let state = SessionState::new("/photos", "/documents");
    let surface =
        phocommander_ui::surface::WorkSurface::new(state.left.clone(), state.right.clone());

    // act
    let links = surface.panel(PanelSide::Left).path().to_path_buf();
    let rechts = surface.panel(PanelSide::Right).path().to_path_buf();

    // assert
    assert_eq!(links, PathBuf::from("/photos"));
    assert_eq!(rechts, PathBuf::from("/documents"));
}

#[test]
fn the_setting_of_the_user_decides_whether_the_session_comes_back() {
    // arrange — a state of the session in a directory of its own.
    let directory = std::env::temp_dir().join(format!(
        "phocommander-sitzung-{:?}",
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    let state = SessionState::new("/photos", "/documents");
    let sicherheit = Security {
        remember_session: true,
        ..Security::default()
    };
    let regeln = phocommander_config::resolve_security(&sicherheit);
    let geschrieben = write_session_state(&directory, &state, &regeln);
    assert!(geschrieben.is_ok(), "{geschrieben:?}");

    // act — the user asks for the session, and the composition reads it.
    let mut config = Config::default();
    config.general.restore_session = true;
    let (links, rechts) =
        phocommander_desktop::composition::starting_directories_in(&config, &directory);

    // assert — the panels show the directories of the last time.
    assert_eq!(links, PathBuf::from("/photos"));
    assert_eq!(rechts, PathBuf::from("/documents"));

    // act — the user does not want the session.
    config.general.restore_session = false;
    let (links, rechts) =
        phocommander_desktop::composition::starting_directories_in(&config, &directory);

    // assert — the panels show the working directory.
    assert_ne!(links, PathBuf::from("/photos"));
    assert_ne!(rechts, PathBuf::from("/documents"));
    let _ = std::fs::remove_dir_all(&directory);
}
