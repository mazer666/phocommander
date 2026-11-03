//! The translation of the commands of the surface into the jobs of the queue.
//!
//! WHAT: the adapter that takes the commands the surface produced and
//!       submits the matching jobs to the waiting room ([`submit`]).
//! WHY: the surface knows what the user asked for, and the operations own
//! how a job runs. The two meet in the adapter, so that neither crate has
//! to know the other: the surface knows no file system, and the operations
//! know no widget.
//! The commands of the surface name the *panels* — the source and the
//! destination — and the adapter has to ask the surface for the
//! directories of those panels. A command that named a path would have to
//! know the state of the panels, and the state belongs to the surface.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::execute::submit;
//! use phocommander_ops::queue::JobQueue;
//! use phocommander_ui::surface::WorkSurface;
//!
//! let queue = JobQueue::new(std::sync::Arc::new(phocommander_ops::fake_fs::FakeFileSystem::new()), 1);
//! let mut surface = WorkSurface::new(std::path::PathBuf::from("/tmp"), std::path::PathBuf::from("/var"));
//! let anzahl = submit(
//!     &[],
//!     &mut surface,
//!     &queue,
//!     &phocommander_ops::fake_fs::FakeFileSystem::new(),
//!     phocommander_ops::copy::CopyMode::Verified,
//!     phocommander_ops::copy::CopyPolicy::strict(),
//! );
//! assert_eq!(anzahl, 0);
//! ```

use phocommander_ops::copy::{CopyMode, CopyPolicy};
use phocommander_ops::job::{Job, JobKind, Priority};
use phocommander_ops::port::FileSystem;
use phocommander_ops::queue::JobQueue;
use phocommander_ui::commands::Command;
use phocommander_ui::surface::{WorkSurface, panel_path};

use crate::listing::read_entries;

/// Submits the commands of the surface to the waiting room.
///
/// WHY: the commands are read once and the jobs are submitted one by one. A
///       command that the adapter does not know is skipped — the surface
///       knows commands that no file operation belongs to, like the command
///       that opens the menu.
pub fn submit(
    commands: &[Command],
    surface: &mut WorkSurface,
    queue: &JobQueue,
    fs: &dyn FileSystem,
    mode: CopyMode,
    rules: CopyPolicy,
) -> usize {
    let mut anzahl = 0;
    for command in commands {
        if submit_one(command, surface, queue, fs, mode, rules) {
            anzahl += 1;
        }
    }
    anzahl
}

/// Submits one command and says whether it became a job.
fn submit_one(
    command: &Command,
    surface: &mut WorkSurface,
    queue: &JobQueue,
    fs: &dyn FileSystem,
    mode: CopyMode,
    rules: CopyPolicy,
) -> bool {
    match command {
        Command::LoadDirectory { side, path } => {
            // WHY: the directory is read here and not in the surface. The
            //       surface knows the panel, and the file system knows the
            //       disk — and the adapter is the one place where the two
            //       meet.
            //
            //       A directory that cannot be read leaves the panel as it
            //       was. An empty panel and a reason in the log are better
            //       than a panel that pretends the directory is empty.
            let Ok(eintraege) = read_entries(fs, path) else {
                return false;
            };
            let panel = surface.panel_mut(*side);
            panel.set_path(path.clone());
            panel.set_entries(eintraege);
            true
        }
        Command::OpenInOtherPanel {
            destination, entry, ..
        } => {
            // WHY: the entry is opened in the other panel, and the other
            //       panel shows its directory. The command names the entry
            //       and the panel, and the adapter asks the file system
            //       what the entry is — a directory is shown in the panel,
            //       and a file is shown in the quick view of the panel.
            let Ok(metadata) = fs.metadata(entry) else {
                return false;
            };
            if metadata.kind.is_directory() {
                let eintraege = match read_entries(fs, entry) {
                    Ok(eintraege) => eintraege,
                    Err(_) => return false,
                };
                let panel = surface.panel_mut(*destination);
                panel.set_path(entry.clone());
                panel.set_entries(eintraege);
            }
            true
        }
        Command::Copy { transfer } => {
            let (quelle, ziel) = directories_of(transfer, surface);
            queue.enqueue(
                job(
                    JobKind::Copy {
                        source: quelle,
                        destination: ziel,
                        mode,
                    },
                    Some(transfer.entries().to_vec()),
                )
                .with_rules(rules),
            );
            true
        }
        Command::Move { transfer } => {
            let (quelle, ziel) = directories_of(transfer, surface);
            queue.enqueue(
                job(
                    JobKind::Move {
                        source: quelle,
                        destination: ziel,
                        mode,
                    },
                    Some(transfer.entries().to_vec()),
                )
                .with_rules(rules),
            );
            true
        }
        Command::Delete { entries, .. } => {
            queue.enqueue(job(
                JobKind::Delete {
                    paths: entries.clone(),
                },
                None,
            ));
            true
        }
        Command::Rename { pairs, .. } => {
            // WHY: the job renames one file, and the command carries the
            //       pairs that the dialog has shown the user (FR-34). A
            //       command with five pairs therefore becomes five jobs,
            //       and the list of the jobs shows the user what is left.
            for pair in pairs {
                queue.enqueue(job(
                    JobKind::Rename {
                        source: pair.source.clone(),
                        destination: pair.target.clone(),
                    },
                    None,
                ));
            }
            true
        }
        Command::CreateDirectory { path, .. } => {
            queue.enqueue(job(JobKind::CreateDirectory { path: path.clone() }, None));
            true
        }
        // WHY: the other commands belong to the surface itself — the menu,
        //       the viewer, the panels. They change no file and therefore
        //       become no job.
        _ => false,
    }
}

/// The directories of the two panels of a transfer.
fn directories_of(
    transfer: &phocommander_ui::surface::TransferRequest,
    surface: &WorkSurface,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let quelle = panel_path(surface.panel(transfer.source())).to_path_buf();
    let ziel = panel_path(surface.panel(transfer.destination())).to_path_buf();
    (quelle, ziel)
}

/// A job with the given work.
fn job(kind: JobKind, restricted_items: Option<Vec<std::path::PathBuf>>) -> Job {
    let mut job = Job::new(kind, Priority::Normal);
    job.restricted_items = restricted_items;
    job
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use phocommander_ops::job::JobKind;
    use phocommander_ui::panel::PanelSide;
    use phocommander_ui::surface::TransferRequest;
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

    /// Hands the commands over to the adapter.
    fn submit_all(
        commands: &[Command],
        surface: &mut WorkSurface,
        queue: &JobQueue,
        fs: &dyn FileSystem,
    ) -> usize {
        super::submit(
            commands,
            surface,
            queue,
            fs,
            CopyMode::Verified,
            CopyPolicy::strict(),
        )
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
        super::submit(
            &commands,
            &mut surface,
            &queue,
            &fs,
            CopyMode::Verified,
            regeln,
        );

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
    fn a_command_that_moves_no_file_becomes_no_job() {
        // arrange
        let (mut surface, queue) = parts();
        let fs = phocommander_ops::fake_fs::FakeFileSystem::new();
        let commands = vec![Command::SwapPanels, Command::Menu];

        // act
        let anzahl = submit_all(&commands, &mut surface, &queue, &fs);

        // assert
        // WHY: the menu and the arrangement of the panels belong to the
        //       surface itself. They change no file, and they become no
        //       job.
        assert_eq!(anzahl, 0);
        assert!(queue.snapshot().is_empty());
    }
}
