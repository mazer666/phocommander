//! The settings of the user decide what the panel shows.
//!
//! WHAT: the test reads a directory that carries a hidden file and a file
//!       of the system, once with the options of the user who wants to see
//!       everything and once with the options of the user who does not.
//! WHY: the user writes in the configuration file whether the hidden files
//!       and the files of the system shall be shown. The setting was read
//!       and never applied — the user wrote the value, saw nothing change,
//!       and concluded that the program is broken. A setting that nobody
//!       applies is worse than no setting (FR-74).
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-desktop --test the_settings_of_the_user_decide
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_desktop::listing::read_entries_with;
use phocommander_ops::fake_fs::FakeFileSystem;
use phocommander_ui::visibility::ListingOptions;

/// A file system that carries the names that matter.
fn file_system() -> FakeFileSystem {
    let fs = FakeFileSystem::new();
    for name in ["notes.txt", ".hidden", ".DS_Store", "Thumbs.db"] {
        fs.write_file(&std::path::Path::new("/photos").join(name), b"x")
            .unwrap_or_else(|error| panic!("the file should be writable: {error:?}"));
    }
    fs
}

/// The names of the entries that the panel gets.
fn namen(fs: &FakeFileSystem, options: ListingOptions) -> Vec<String> {
    let eintraege = read_entries_with(fs, std::path::Path::new("/photos"), options)
        .unwrap_or_else(|error| panic!("the directory should be readable: {error:?}"));
    eintraege.into_iter().map(|eintrag| eintrag.name).collect()
}

#[test]
fn the_user_who_wants_everything_sees_everything() {
    // arrange
    let fs = file_system();

    // act
    let gesehen = namen(&fs, ListingOptions::ALL);

    // assert
    assert_eq!(gesehen.len(), 4, "{gesehen:?}");
    assert!(gesehen.contains(&".hidden".to_string()), "{gesehen:?}");
    assert!(gesehen.contains(&".DS_Store".to_string()), "{gesehen:?}");
}

#[test]
fn the_user_who_wants_the_ordinary_files_only_sees_them() {
    // arrange
    let fs = file_system();

    // act
    let gesehen = namen(&fs, ListingOptions::ORDINARY);

    // assert — the hidden file and the files of the system stay out.
    assert_eq!(gesehen, vec!["notes.txt".to_string()], "{gesehen:?}");
}

#[test]
fn the_user_who_wants_the_hidden_files_still_leaves_the_system_out() {
    // arrange
    let fs = file_system();

    // act
    let gesehen = namen(
        &fs,
        ListingOptions {
            show_hidden: true,
            show_system: false,
        },
    );

    // assert
    assert!(gesehen.contains(&".hidden".to_string()), "{gesehen:?}");
    assert!(!gesehen.contains(&".DS_Store".to_string()), "{gesehen:?}");
    assert!(!gesehen.contains(&"Thumbs.db".to_string()), "{gesehen:?}");
}

#[test]
fn the_setting_of_the_user_reaches_the_shell() {
    // arrange — a configuration in which the user does not want to see the
    // hidden files.
    let mut config = phocommander_config::Config::default();
    config.appearance.show_hidden_files = false;
    config.appearance.show_system_files = false;

    // act — the application is built out of the configuration.
    let app = phocommander_desktop::composition::build_application(&config);

    // assert — the shell carries the wish of the user, and the adapter
    // asks the shell before it reads a directory.
    let visibility = app.surface().visibility();
    assert!(
        !visibility.show_hidden,
        "the shell should hide the hidden files"
    );
    assert!(
        !visibility.show_system,
        "the shell should hide the files of the system"
    );
}

#[test]
fn the_way_of_the_synchronisation_comes_from_the_user() {
    // arrange — a configuration in which the user wants the target to
    // become the mirror of the source.
    let mut config = phocommander_config::Config::default();
    config.copy.sync_mode = phocommander_config::SyncMode::SyncMirror;

    // act — the application is built out of the configuration.
    let app = phocommander_desktop::composition::build_desktop_app(&config);

    // assert — the way of the synchronisation reached the shell.
    assert_eq!(
        app.sync_mode(),
        phocommander_ops::copy::CopyMode::SyncMirror,
        "the way of the synchronisation should come from the user"
    );
}
