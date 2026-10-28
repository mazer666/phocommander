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
//! let surface = WorkSurface::new(std::path::PathBuf::from("/tmp"), std::path::PathBuf::from("/var"));
//! let anzahl = submit(&[], &surface, &queue, phocommander_ops::copy::CopyMode::Verified);
//! assert_eq!(anzahl, 0);
//! ```

use phocommander_ops::copy::CopyMode;
use phocommander_ops::job::{Job, JobKind, Priority};
use phocommander_ops::queue::JobQueue;
use phocommander_ui::commands::Command;
use phocommander_ui::surface::{WorkSurface, panel_path};

/// Submits the commands of the surface to the waiting room.
///
/// WHY: the commands are read once and the jobs are submitted one by one. A
///       command that the adapter does not know is skipped — the surface
///       knows commands that no file operation belongs to, like the command
///       that opens the menu.
pub fn submit(
    commands: &[Command],
    surface: &WorkSurface,
    queue: &JobQueue,
    mode: CopyMode,
) -> usize {
    let mut anzahl = 0;
    for command in commands {
        if submit_one(command, surface, queue, mode) {
            anzahl += 1;
        }
    }
    anzahl
}

/// Submits one command and says whether it became a job.
fn submit_one(command: &Command, surface: &WorkSurface, queue: &JobQueue, mode: CopyMode) -> bool {
    match command {
        Command::Copy { transfer } => {
            let (quelle, ziel) = directories_of(transfer, surface);
            queue.enqueue(job(
                JobKind::Copy {
                    source: quelle,
                    destination: ziel,
                    mode,
                },
                Some(transfer.entries().to_vec()),
            ));
            true
        }
        Command::Move { transfer } => {
            let (quelle, ziel) = directories_of(transfer, surface);
            queue.enqueue(job(
                JobKind::Move {
                    source: quelle,
                    destination: ziel,
                    mode,
                },
                Some(transfer.entries().to_vec()),
            ));
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

    /// A surface with two panels and a waiting room.
    fn parts() -> (WorkSurface, JobQueue) {
        let surface = WorkSurface::new(
            std::path::PathBuf::from("/photos"),
            std::path::PathBuf::from("/backup"),
        );
        let queue = JobQueue::new(
            std::sync::Arc::new(phocommander_ops::fake_fs::FakeFileSystem::new()),
            1,
        );
        (surface, queue)
    }

    #[test]
    fn the_command_of_a_copy_becomes_a_job_of_the_waiting_room() {
        // arrange
        let (surface, queue) = parts();
        let transfer = TransferRequest::new(
            PanelSide::Left,
            PanelSide::Right,
            vec![std::path::PathBuf::from("/photos/a.txt")],
        )
        .unwrap_or_else(|error| panic!("the transfer should be valid: {error}"));
        let commands = vec![Command::Copy {
            transfer: transfer.clone(),
        }];

        // act
        let anzahl = submit(&commands, &surface, &queue, CopyMode::Verified);

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
                assert_eq!(source, std::path::Path::new("/photos"));
                assert_eq!(destination, std::path::Path::new("/backup"));
                assert_eq!(*mode, CopyMode::Verified);
            }
            other => panic!("the job should be a copy: {other:?}"),
        }
    }

    #[test]
    fn the_command_of_a_copy_carries_the_entries_that_were_marked() {
        // arrange
        let (surface, queue) = parts();
        let transfer = TransferRequest::new(
            PanelSide::Left,
            PanelSide::Right,
            vec![
                std::path::PathBuf::from("/photos/a.txt"),
                std::path::PathBuf::from("/photos/b.txt"),
            ],
        )
        .unwrap_or_else(|error| panic!("the transfer should be valid: {error}"));
        let commands = vec![Command::Copy { transfer }];

        // act
        submit(&commands, &surface, &queue, CopyMode::Quick);

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
        let (surface, queue) = parts();
        let commands = vec![Command::Delete {
            side: PanelSide::Left,
            entries: vec![std::path::PathBuf::from("/photos/a.txt")],
        }];

        // act
        let anzahl = submit(&commands, &surface, &queue, CopyMode::Verified);

        // assert
        assert_eq!(anzahl, 1);
        let snapshot = queue.snapshot();
        match &snapshot[0].kind {
            JobKind::Delete { paths } => assert_eq!(paths.len(), 1),
            other => panic!("the job should be a deletion: {other:?}"),
        }
    }

    #[test]
    fn the_key_of_the_user_becomes_a_job_of_the_waiting_room() {
        // arrange
        // WHY: the whole chain in one test — the key of the user, the
        //       action of the table, the command of the surface and the job
        //       of the waiting room. A gap in any link of the chain means
        //       that the application copies nothing, and each link has its
        //       own test. This one proves that the chain holds together.
        let (surface, queue) = parts();
        let mut keymap = std::collections::BTreeMap::new();
        keymap.insert("panel.copy".to_string(), "F5".to_string());
        let mut tables = std::collections::BTreeMap::new();
        tables.insert(phocommander_ui::context::Context::Keyboard, keymap);
        let keymap = phocommander_ui::keymap_view::KeymapView::from_context_tables(&tables)
            .unwrap_or_else(|error| panic!("the tables should be valid: {error}"));

        let mut app =
            phocommander_ui::WorkSurfaceApp::new(surface, keymap, Box::new(EmptyCatalogue));
        // WHY: the user marks the entries and presses the key. The marking
        //       happens on the tab of the panel.
        app.surface_mut()
            .panel_mut(phocommander_ui::panel::PanelSide::Left)
            .active_tab_mut()
            .select_all();
        let transfer = app
            .surface()
            .transfer_to_other()
            .unwrap_or_else(|error| panic!("the transfer should be valid: {error}"));
        let commands = vec![Command::Copy { transfer }];

        // act
        let anzahl = submit(&commands, app.surface(), &queue, CopyMode::Verified);

        // assert
        assert_eq!(anzahl, 1, "the key should have become a job");
        assert_eq!(queue.snapshot().len(), 1);
    }

    /// The catalogue that knows no key.
    struct EmptyCatalogue;

    impl phocommander_ui::Catalogue for EmptyCatalogue {
        fn lookup_with(
            &self,
            _key: &str,
            _arguments: &[phocommander_ui::Argument<'_>],
        ) -> Option<String> {
            None
        }
    }

    #[test]
    fn a_command_that_moves_no_file_becomes_no_job() {
        // arrange
        let (surface, queue) = parts();
        let commands = vec![Command::SwapPanels, Command::Menu];

        // act
        let anzahl = submit(&commands, &surface, &queue, CopyMode::Verified);

        // assert
        // WHY: the menu and the arrangement of the panels belong to the
        //       surface itself. They change no file, and they become no
        //       job.
        assert_eq!(anzahl, 0);
        assert!(queue.snapshot().is_empty());
    }
}
