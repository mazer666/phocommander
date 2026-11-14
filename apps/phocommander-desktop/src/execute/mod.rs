//! The translation of the commands of the surface into the jobs of the adapter.queue.
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
//! let viewer = phocommander_desktop::preview::ViewerPreviewSource::new(
//!     phocommander_viewer::reader::ViewerReader::new(
//!         phocommander_viewer::policy::ReadPolicy::strict(),
//!         10_485_760,
//!     ),
//! );
//! let mut undo = phocommander_ops::undo::UndoLedger::new();
//! let mut adapter = phocommander_desktop::execute::Adapter {
//!     queue: &queue,
//!     fs: &phocommander_ops::fake_fs::FakeFileSystem::new(),
//!     viewer: &viewer,
//!     ways: phocommander_desktop::execute::Ways::SAFE,
//!     rules: phocommander_ops::copy::CopyPolicy::strict(),
//!     undo: &mut undo,
//! };
//! let anzahl = submit(&[], &mut surface, &mut adapter);
//! assert_eq!(anzahl, 0);
//! ```

use phocommander_ops::copy::{CopyMode, CopyPolicy};
use phocommander_ops::job::{Job, JobKind, Priority};
use phocommander_ops::port::FileSystem;
use phocommander_ops::queue::JobQueue;
use phocommander_ops::undo::UndoLedger;
use phocommander_ui::commands::Command;
use phocommander_ui::panel::ArchiveView;
use phocommander_ui::surface::{WorkSurface, panel_path};

use crate::archive::{entries_of_the_archive, entries_of_the_archive_at};
use crate::listing::read_entries_with;
use crate::preview::ViewerPreviewSource;

/// The things the adapter needs to carry out a command.
///
/// WHY: the adapter reaches the file system, the waiting room and the
///       ledger of the adapter.undo. A command needs all of them, and a list of
///       eight parameters says less than one structure that names them.
pub struct Adapter<'a> {
    /// The waiting room that carries the jobs out.
    pub queue: &'a JobQueue,

    /// The file system of the host.
    pub fs: &'a dyn FileSystem,

    /// The reader of the quick view.
    pub viewer: &'a ViewerPreviewSource,

    /// The ways in which the files are copied and synchronised.
    pub ways: Ways,

    /// The rules of the copy.
    pub rules: CopyPolicy,

    /// What has been copied and can be taken back.
    pub undo: &'a mut UndoLedger,
}

/// The scope of the undo, in the words of the interface.
///
/// WHY: the operations crate and the interface crate have their own scope,
///       because no crate below `crates/` may depend on a sibling
///       (ADR-0001, option B). The adapter is the place where the two
///       meet, and it translates the one into the other (FR-26).
fn scope_of(scope: phocommander_ops::UndoScope) -> phocommander_ui::undo::UndoScope {
    match scope {
        phocommander_ops::UndoScope::RemovesCopiesOnly => {
            phocommander_ui::undo::UndoScope::RemovesCopiesOnly
        }
    }
}

/// The ways in which the files are copied and synchronised.
///
/// WHY: the two ways belong together — both come from the configuration
///       of the user, and both decide what a job does. A pair of values in
///       one structure is easier to pass on than eight parameters, and the
///       call reads as it means: "these ways, these rules" (FR-22b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ways {
    /// The way a file is copied.
    pub copy: CopyMode,

    /// The way a directory is synchronised.
    pub sync: CopyMode,
}

impl Ways {
    /// The ways of a user who wants every copy verified and every
    /// synchronisation shown as a plan.
    pub const SAFE: Ways = Ways {
        copy: CopyMode::Verified,
        sync: CopyMode::SyncPreview,
    };
}

/// Submits the commands of the surface to the waiting room.
///
/// WHY: the commands are read once and the jobs are submitted one by one. A
///       command that the adapter does not know is skipped — the surface
///       knows commands that no file operation belongs to, like the command
///       that opens the menu.
pub fn submit(commands: &[Command], surface: &mut WorkSurface, adapter: &mut Adapter<'_>) -> usize {
    let mut anzahl = 0;
    for command in commands {
        if submit_one(command, surface, adapter) {
            anzahl += 1;
        }
    }
    anzahl
}

/// Submits one command and says whether it became a job.
fn submit_one(command: &Command, surface: &mut WorkSurface, adapter: &mut Adapter<'_>) -> bool {
    match command {
        Command::OpenEntry {
            side,
            path,
            archive,
        } => {
            // WHY: the surface knows the name of the entry and not its kind.
            //       The shell decides: a directory is read, and an archive
            //       opens as a virtual directory (FR-65).
            if let Some(view) = archive {
                // WHY: the entry lies inside the archive. The place below
                //       the entry is the place that the panel shows.
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default();
                let innen = view.below(&name);
                let panel = surface.panel_mut(*side);
                panel.set_archive(innen.clone());
                let eintraege = entries_of_the_archive(&innen);
                panel.set_entries(eintraege);
                return true;
            }

            // WHY: an archive opens as a virtual directory. The container
            //       decides itself whether it can be opened, and the entries
            //       of its root fill the panel (FR-65). The archive comes
            //       first, because the container knows a file that the
            //       file system of the host does not — an archive is a
            //       file of the disk, and the host knows it as well.
            let eintraege = entries_of_the_archive_at(path, "");
            if !eintraege.is_empty() {
                let panel = surface.panel_mut(*side);
                panel.set_archive(ArchiveView::new(path.clone()));
                panel.set_entries(eintraege);
                return true;
            }

            let Ok(metadata) = adapter.fs.metadata(path) else {
                return false;
            };
            if !metadata.kind.is_directory() {
                // WHY: a file that is no archive is no directory. The
                //       viewer opens it, and the panel keeps what it shows.
                return false;
            }
            let Ok(eintraege) = read_entries_with(adapter.fs, path, surface.visibility()) else {
                return false;
            };
            let panel = surface.panel_mut(*side);
            panel.set_path(path.clone());
            panel.set_entries(eintraege);
            true
        }
        Command::Deferred { action } => {
            // WHY: the user pressed a key and the program cannot do it yet.
            //       The shell has already said so in the status line, and
            //       the notice names the action — a key that does nothing
            //       without a word looks like a broken key (FR-05, FR-124).
            //       The shell is the only place that knows the language of
            //       the interface, and therefore the only place that can
            //       write the name of the action in that language.
            let _ = action;
            true
        }
        Command::Sync { side } => {
            // WHY: the synchronisation compares the two directories and
            //       brings the one into the other. The plan is built here,
            //       because the adapter knows the file system — and the
            //       way of the synchronisation comes from the setting of
            //       the user (FR-22b).
            let quelle = panel_path(surface.panel(*side)).to_path_buf();
            let ziel = panel_path(surface.panel(side.other())).to_path_buf();
            let plan = match phocommander_ops::sync::build_plan(
                adapter.fs,
                &quelle,
                &ziel,
                adapter.ways.sync,
            ) {
                Ok(plan) => plan,
                Err(error) => {
                    // WHY: a directory that cannot be compared is not an
                    //       error that stops the application — the panels
                    //       keep what they show.
                    eprintln!("the comparison of the two directories failed: {error}");
                    return false;
                }
            };
            // WHY: the preview of the synchronisation shows the plan of the
            //       comparison, and the plan is a document that the user can
            //       read. The preview of the mode "sync-preview" therefore
            //       goes into the area of the viewer, and the user sees
            //       what the synchronisation would do before anything
            //       happens (FR-22b).
            if matches!(adapter.ways.sync, CopyMode::SyncPreview) {
                let document = plan.to_document();
                let zeilen: Vec<String> = document.lines().map(str::to_string).collect();
                let groesse = document.len() as u64;
                surface.set_viewer(phocommander_ui::preview::Preview::new(
                    ziel.clone(),
                    groesse,
                    zeilen,
                    false,
                ));
            }
            let job = phocommander_ops::executor::confirmed_job(
                plan,
                phocommander_ops::Approval::Granted,
                Priority::Normal,
            );
            match job {
                Ok(job) => {
                    adapter.queue.enqueue(job.with_rules(adapter.rules));
                }
                Err(error) => {
                    eprintln!("the job of the synchronisation was refused: {error}");
                    return false;
                }
            }
            true
        }
        Command::Undo => {
            // WHY: the undo takes the last copy back. The ledger knows
            //       what was copied, and the scope of the undo is partial:
            //       the copies go, the state before them does not come
            //       back (FR-26).
            let outcome = match adapter.undo.undo_last(adapter.fs) {
                Ok(outcome) => outcome,
                Err(error) => {
                    eprintln!("the undo failed: {error}");
                    return false;
                }
            };
            // WHY: the surface shows the notice of the undo area as long
            //       as an undo is on offer. An empty ledger means that
            //       there is nothing left to take back.
            surface.set_undo(if adapter.undo.is_empty() {
                phocommander_ui::undo::UndoAvailability::Unavailable
            } else {
                phocommander_ui::undo::UndoAvailability::available(scope_of(adapter.undo.scope()))
            });
            let removed = outcome.removed.len();
            surface.set_status(phocommander_ui::surface::StatusMessage {
                key: "panel-undo-done".to_string(),
                arguments: vec![("count", removed.to_string())],
            });
            true
        }
        Command::SwapPanels => {
            // WHY: the user exchanges the two panels. The surface knows
            //       the panels, and the surface carries out the exchange
            //       (FR-11).
            surface.swap_panels();
            true
        }
        Command::OpenViewer { entry, .. } => {
            // WHY: the viewer opens in an additional area of the surface and
            //       not in a window of its own, so that the user does not
            //       leave the application to look at a file (FR-50). The
            //       content comes from the reader of the viewer, and it
            //       travels to the area that shows it.
            let preview = match adapter.viewer.open(entry) {
                Some(preview) => preview,
                None => return false,
            };
            surface.set_viewer(preview);
            true
        }
        Command::ExtractFromArchive {
            archive,
            entry,
            destination,
        } => {
            // WHY: the entry is taken out of the archive into the other
            //       panel. Everything else stays in the archive, and the
            //       extraction carries the limits and the rules of the
            //       security section (FR-66).
            let ziel = surface.panel(*destination).path().to_path_buf();
            let options = phocommander_archive::ExtractionOptions::default();
            match phocommander_archive::extract_entry(archive, entry, &ziel, &options) {
                Ok(report) => report.written_entries > 0,
                Err(_) => false,
            }
        }
        Command::LoadDirectory { side, path } => {
            // WHY: the directory is read here and not in the surface. The
            //       surface knows the panel, and the file system knows the
            //       disk — and the adapter is the one place where the two
            //       meet.
            //
            //       A directory that cannot be read leaves the panel as it
            //       was. An empty panel and a reason in the log are better
            //       than a panel that pretends the directory is empty.
            let Ok(eintraege) = read_entries_with(adapter.fs, path, surface.visibility()) else {
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
            let Ok(metadata) = adapter.fs.metadata(entry) else {
                return false;
            };
            if metadata.kind.is_directory() {
                let eintraege = match read_entries_with(adapter.fs, entry, surface.visibility()) {
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
            // WHY: the undo takes a copy back, and it can only take back
            //       what it knows. The adapter records the copy before it
            //       hands it to the waiting room (FR-26).
            let erzeugt: Vec<std::path::PathBuf> = transfer
                .entries()
                .iter()
                .map(|name| ziel.join(name))
                .collect();
            adapter
                .undo
                .record_copy(quelle.clone(), ziel.clone(), erzeugt);
            surface.set_undo(phocommander_ui::undo::UndoAvailability::available(
                scope_of(adapter.undo.scope()),
            ));
            adapter.queue.enqueue(
                job(
                    JobKind::Copy {
                        source: quelle,
                        destination: ziel,
                        mode: adapter.ways.copy,
                    },
                    Some(transfer.entries().to_vec()),
                )
                .with_rules(adapter.rules),
            );
            true
        }
        Command::Move { transfer } => {
            let (quelle, ziel) = directories_of(transfer, surface);
            adapter.queue.enqueue(
                job(
                    JobKind::Move {
                        source: quelle,
                        destination: ziel,
                        mode: adapter.ways.copy,
                    },
                    Some(transfer.entries().to_vec()),
                )
                .with_rules(adapter.rules),
            );
            true
        }
        Command::Delete { entries, .. } => {
            adapter.queue.enqueue(job(
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
                adapter.queue.enqueue(job(
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
            adapter
                .queue
                .enqueue(job(JobKind::CreateDirectory { path: path.clone() }, None));
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
// WHY: the tests of the adapter live in a file of their own, so that the
//       adapter itself stays readable.
mod tests;
