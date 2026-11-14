//! The tests of the adapter that carries the commands of the surface out.
//!
//! WHY: the tests live in a file of their own, so that the adapter itself
//!       stays readable. The rule that forbids a panic applies to the
//!       production code, and it is enforced there.
use crate::execute::{Adapter, Ways};
use crate::preview::ViewerPreviewSource;
use phocommander_ops::copy::{CopyMode, CopyPolicy};
use phocommander_ops::job::JobKind;
use phocommander_ops::port::FileSystem;
use phocommander_ops::queue::JobQueue;
use phocommander_ops::undo::UndoLedger;
use phocommander_ui::commands::Command;
use phocommander_ui::panel::PanelSide;
use phocommander_ui::surface::TransferRequest;
use phocommander_ui::surface::WorkSurface;
use phocommander_viewer::policy::ReadPolicy;
use phocommander_viewer::reader::ViewerReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A surface with two panels and a waiting room.
fn parts() -> (WorkSurface, JobQueue) {
    let surface = WorkSurface::new(PathBuf::from("/photos"), PathBuf::from("/backup"));
    let queue = JobQueue::new(
        Arc::new(phocommander_ops::fake_fs::FakeFileSystem::new()),
        1,
    );
    (surface, queue)
}

/// Builds a ZIP archive with the given entries.
///
/// WHY: the archive is a real file on the disk. The reader of the
///       container reads it, and the panel shows what it carries.
fn build_zip(path: &std::path::Path, entries: &[(&str, &[u8])]) {
    use phocommander_archive::formats::ArchiveWriter;
    use phocommander_archive::formats::zip::ZipArchiveWriter;

    let mut writer = ZipArchiveWriter::create(path)
        .unwrap_or_else(|error| panic!("the archive should have been created: {error}"));
    for (name, content) in entries {
        writer
            .add_file(
                name,
                content.len() as u64,
                &mut std::io::Cursor::new(content.to_vec()),
            )
            .unwrap_or_else(|error| panic!("the entry should have been added: {error}"));
    }
    Box::new(writer)
        .finish()
        .unwrap_or_else(|error| panic!("the archive should have been closed: {error}"));
}

/// Hands the commands over to the adapter.
fn submit_all(
    commands: &[Command],
    surface: &mut WorkSurface,
    queue: &JobQueue,
    fs: &dyn FileSystem,
) -> usize {
    let viewer = Box::new(ViewerPreviewSource::new(ViewerReader::new(
        ReadPolicy::strict(),
        10_485_760,
    )));
    let mut undo = UndoLedger::new();
    let mut adapter = Adapter {
        queue,
        fs,
        viewer: viewer.as_ref(),
        ways: Ways::SAFE,
        rules: CopyPolicy::strict(),
        undo: &mut undo,
    };
    crate::execute::submit(commands, surface, &mut adapter)
}

#[test]
fn the_command_of_a_copy_becomes_a_job_of_the_waiting_room() {
    // arrange
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let transfer = TransferRequest::new(
        PanelSide::Left,
        PanelSide::Right,
        vec![PathBuf::from("/photos/a.txt")],
    )
    .unwrap_or_else(|error| panic!("the transfer should be valid: {error}"));
    let commands = vec![Command::Copy {
        transfer: transfer.clone(),
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    // WHY: the command of the surface is a request of the user, and the
    //       job of the waiting room is what carries it out. Without this
    //       translation the copy would never happen.
    assert_eq!(anzahl, 1);
    let snapshot = queue.snapshot();
    assert_eq!(snapshot.len(), 1);
    match &snapshot[0].kind {
        JobKind::Copy {
            source,
            destination,
            mode,
        } => {
            assert_eq!(source, Path::new("/photos"));
            assert_eq!(destination, Path::new("/backup"));
            assert_eq!(*mode, CopyMode::Verified);
        }
        other => panic!("the job should be a copy: {other:?}"),
    }
}

#[test]
fn the_command_of_a_copy_carries_the_entries_that_were_marked() {
    // arrange
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let transfer = TransferRequest::new(
        PanelSide::Left,
        PanelSide::Right,
        vec![
            PathBuf::from("/photos/a.txt"),
            PathBuf::from("/photos/b.txt"),
        ],
    )
    .unwrap_or_else(|error| panic!("the transfer should be valid: {error}"));
    let commands = vec![Command::Copy { transfer }];

    // act
    submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    // WHY: the copy is limited to the entries the user marked, and not
    //       to the whole directory.
    let snapshot = queue.snapshot();
    let items = snapshot[0]
        .restricted_items
        .clone()
        .unwrap_or_else(|| panic!("the job should be limited to its entries"));
    assert_eq!(items.len(), 2);
}

#[test]
fn the_command_of_a_deletion_becomes_a_job() {
    // arrange
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let commands = vec![Command::Delete {
        side: PanelSide::Left,
        entries: vec![PathBuf::from("/photos/a.txt")],
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    assert_eq!(anzahl, 1);
    let snapshot = queue.snapshot();
    match &snapshot[0].kind {
        JobKind::Delete { paths } => assert_eq!(paths.len(), 1),
        other => panic!("the job should be a deletion: {other:?}"),
    }
}

#[test]
fn the_job_carries_the_rules_of_the_user() {
    // arrange
    // WHY: the rules decide whether the copy compares the checksums and
    //       whether a link is dereferenced. They come from the
    //       configuration of the user, and a program that decided them
    //       on its own would compare the checksums on a machine where
    //       the user has switched the comparison off.
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let transfer = TransferRequest::new(
        PanelSide::Left,
        PanelSide::Right,
        vec![PathBuf::from("/photos/a.txt")],
    )
    .unwrap_or_else(|error| panic!("the transfer should be valid: {error}"));
    let commands = vec![Command::Copy { transfer }];
    let regeln = CopyPolicy::new(false, false);

    // act
    let viewer = Box::new(ViewerPreviewSource::new(ViewerReader::new(
        ReadPolicy::strict(),
        10_485_760,
    )));
    let mut undo = UndoLedger::new();
    let mut adapter = Adapter {
        queue: &queue,
        fs: &fs,
        viewer: viewer.as_ref(),
        ways: Ways::SAFE,
        rules: regeln,
        undo: &mut undo,
    };
    crate::execute::submit(&commands, &mut surface, &mut adapter);

    // assert
    let snapshot = queue.snapshot();
    assert!(!snapshot[0].rules.verifies_checksums());
    assert!(!snapshot[0].rules.follows_symlinks());
}

#[test]
fn the_command_of_a_directory_fills_the_panel_with_its_entries() {
    // arrange
    // WHY: the directory is the one command that does not become a
    //       job — it fills the panel of the surface. Without it the
    //       panels stay empty, and a file manager with empty panels
    //       shows nothing at all.
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    fs.write_file(Path::new("/photos/a.txt"), b"payload")
        .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
    let commands = vec![Command::LoadDirectory {
        side: PanelSide::Left,
        path: PathBuf::from("/photos"),
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    assert_eq!(anzahl, 1);
    // WHY: the panel shows what the directory holds.
    let panel = surface.panel(PanelSide::Left);
    assert_eq!(panel.path(), Path::new("/photos"));
    assert_eq!(panel.row_count(), 1, "the panel should show the file");
}

#[test]
fn a_directory_that_cannot_be_read_leaves_the_panel_alone() {
    // arrange
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let commands = vec![Command::LoadDirectory {
        side: PanelSide::Left,
        path: PathBuf::from("/gibt-es-nicht"),
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    // WHY: an empty panel and a reason in the log are better than a
    //       panel that pretends the directory is empty.
    assert_eq!(anzahl, 0);
    assert_eq!(surface.panel(PanelSide::Left).path(), Path::new("/photos"));
}

#[test]
fn the_command_of_the_other_panel_opens_the_directory_there() {
    // arrange
    // WHY: the hand-over to the other panel is explicit (FR-16), and it
    //       is the way the user moves in the file system without the
    //       keyboard.
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    fs.create_dir_all(Path::new("/photos/2024"))
        .unwrap_or_else(|error| panic!("the directory should have been created: {error}"));
    fs.write_file(Path::new("/photos/2024/a.txt"), b"payload")
        .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
    let commands = vec![Command::OpenInOtherPanel {
        source: PanelSide::Left,
        destination: PanelSide::Right,
        entry: PathBuf::from("/photos/2024"),
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    assert_eq!(anzahl, 1);
    let panel = surface.panel(PanelSide::Right);
    assert_eq!(panel.path(), Path::new("/photos/2024"));
    assert_eq!(panel.row_count(), 1);
}

#[test]
fn the_command_of_an_archive_fills_the_panel_with_its_entries() {
    // arrange
    // WHY: an archive opens as a virtual directory (FR-65). The panel
    //       shows the entries of the archive, and the user can mark one
    //       of them and take it out.
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let archive = std::env::temp_dir().join(format!(
        "phocommander-virtual-directory-{}.zip",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&archive);
    build_zip(&archive, &[("notes.txt", b"der Inhalt")]);

    let commands = vec![Command::OpenEntry {
        side: PanelSide::Left,
        path: archive.clone(),
        archive: None,
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    assert_eq!(anzahl, 1);
    let panel = surface.panel(PanelSide::Left);
    assert_eq!(panel.row_count(), 1, "the panel should show the entry");
    // WHY: the panel remembers that it shows an archive, and the path of
    //       the archive is the way out of it.
    let view = panel
        .archive()
        .unwrap_or_else(|| panic!("the panel should show an archive"));
    assert_eq!(view.archive(), archive.as_path());
    assert!(view.at_root());

    let _ = std::fs::remove_file(&archive);
}

#[test]
fn the_entry_of_an_archive_is_taken_out_into_the_other_panel() {
    // arrange
    // WHY: the user browses the content of an archive and takes out the
    //       one file that he wants — everything else stays in the
    //       archive (FR-66).
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    // WHY: the name of the file carries the number of the process, so
    //       that two runs of the tests never work on the same file.
    //       The test was flaky: it removed the target of another run.
    let eigen = format!("phocommander-take-out-{}", std::process::id());
    let archive = std::env::temp_dir().join(format!("{eigen}.zip"));
    let ziel = std::env::temp_dir().join(format!("{eigen}-ziel"));
    let _ = std::fs::remove_file(&archive);
    let _ = std::fs::remove_dir_all(&ziel);
    build_zip(&archive, &[("notes.txt", b"der Inhalt der Notizen")]);
    std::fs::create_dir_all(&ziel)
        .unwrap_or_else(|error| panic!("the target should have been created: {error}"));
    // WHY: the other panel shows the place the entry is taken out into.
    surface.panel_mut(PanelSide::Right).set_path(ziel.clone());

    let commands = vec![Command::ExtractFromArchive {
        archive: archive.clone(),
        entry: "notes.txt".to_string(),
        destination: PanelSide::Right,
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    assert_eq!(anzahl, 1);
    // WHY: the file lies in the other panel, and the archive keeps the
    //       rest.
    assert!(
        ziel.join("notes.txt").exists(),
        "the file should be taken out"
    );
    let _ = std::fs::remove_file(&archive);
    let _ = std::fs::remove_dir_all(&ziel);
}

#[test]
fn the_command_of_the_viewer_puts_the_file_into_the_area() {
    // arrange
    // WHY: the viewer opens in an additional area of the surface and
    //       not in a window of its own, so that the user does not leave
    //       the application to look at a file (FR-50).
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let datei = std::env::temp_dir().join("phocommander-viewer-area.txt");
    let _ = std::fs::write(&datei, b"hallo welt");
    let commands = vec![Command::OpenViewer {
        side: PanelSide::Left,
        entry: datei.clone(),
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    assert_eq!(anzahl, 1);
    let viewer = surface
        .viewer()
        .unwrap_or_else(|| panic!("the viewer should show the file"));
    assert_eq!(viewer.lines(), &["hallo welt".to_string()]);
    let _ = std::fs::remove_file(&datei);
}

#[test]
fn a_deferred_action_is_carried_out_without_a_job() {
    // arrange
    // WHY: the user presses a key and the program cannot do it yet. The
    //       shell says so in the status line — in the language of the
    //       interface, and the shell is the only place that knows it.
    //       The adapter therefore carries the command out and starts
    //       no job.
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let action = phocommander_ui::actions::Action::from_id("palette.open")
        .unwrap_or_else(|| panic!("the action should be known"));
    let commands = vec![Command::Deferred { action }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert — the command is carried out, and it starts no job.
    assert_eq!(anzahl, 1);
    assert!(
        queue.snapshot().is_empty(),
        "a deferred action starts no job"
    );
}

#[test]
fn the_command_of_the_synchronisation_becomes_a_job() {
    // arrange — two directories with a file in the one and nothing in
    // the other.
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    fs.write_file(Path::new("/photos/a.txt"), b"hallo")
        .unwrap_or_else(|error| panic!("the file should be writable: {error:?}"));
    fs.write_file(Path::new("/backup/b.txt"), b"welt")
        .unwrap_or_else(|error| panic!("the file should be writable: {error:?}"));
    let commands = vec![Command::Sync {
        side: phocommander_ui::panel::PanelSide::Left,
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert — the comparison of the two directories became a job of
    // the waiting room. The synchronisation was reachable, and this is
    // the proof (FR-22b).
    assert_eq!(anzahl, 1);
    let jobs = queue.snapshot();
    assert_eq!(jobs.len(), 1, "{jobs:?}");
    assert!(
        matches!(jobs[0].kind, JobKind::Sync { .. }),
        "the job should be the synchronisation: {jobs:?}"
    );
}

#[test]
fn the_preview_of_the_synchronisation_shows_the_plan() {
    // arrange — two directories with a file in the one and nothing in
    // the other.
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    fs.write_file(Path::new("/photos/a.txt"), b"hallo")
        .unwrap_or_else(|error| panic!("the file should be writable: {error:?}"));
    let commands = vec![Command::Sync {
        side: phocommander_ui::panel::PanelSide::Left,
    }];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert — the plan of the comparison stands in the area of the
    // viewer, and the user sees what the synchronisation would do
    // (FR-22b).
    assert_eq!(anzahl, 1);
    let viewer = surface
        .viewer()
        .unwrap_or_else(|| panic!("the plan should stand in the viewer"));
    let text = viewer.lines().join("\n");
    assert!(
        text.contains("phocommander-sync-plan"),
        "the plan should be shown: {text}"
    );
}

#[test]
fn the_command_of_the_swap_exchanges_the_panels() {
    // arrange
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    surface
        .panel_mut(phocommander_ui::panel::PanelSide::Left)
        .set_path("/left".into());
    surface
        .panel_mut(phocommander_ui::panel::PanelSide::Right)
        .set_path("/right".into());
    let commands = vec![Command::SwapPanels];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert — the panels have exchanged their places.
    assert_eq!(anzahl, 1);
    assert_eq!(
        surface
            .panel(phocommander_ui::panel::PanelSide::Left)
            .path(),
        std::path::Path::new("/right")
    );
    assert_eq!(
        surface
            .panel(phocommander_ui::panel::PanelSide::Right)
            .path(),
        std::path::Path::new("/left")
    );
}

#[test]
fn a_command_that_moves_no_file_becomes_no_job() {
    // arrange
    let (mut surface, queue) = parts();
    let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
    let commands = vec![Command::SwapPanels];

    // act
    let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

    // assert
    // WHY: the exchange of the panels belongs to the surface itself.
    //       It changes no file, and it becomes no job.
    assert_eq!(anzahl, 1);
    assert!(queue.snapshot().is_empty());
}
