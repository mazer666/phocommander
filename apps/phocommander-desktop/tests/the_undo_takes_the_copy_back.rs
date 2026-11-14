//! The undo takes the last copy back.
//!
//! WHAT: the test copies a file, takes the copy back and reads the state
//!       of the undo area.
//! WHY: the undo of the specification exists for the copy operations and is
//!       marked as partial: it removes the copies and restores nothing
//!       (FR-26). The ledger of the undo existed as a library, and no key
//!       reached it — the user copied a file and could not take it back.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-desktop --test the_undo_takes_the_copy_back
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_desktop::execute::{Adapter, Ways};
use phocommander_desktop::preview::ViewerPreviewSource;
use phocommander_ops::copy::CopyPolicy;
use phocommander_ops::fake_fs::FakeFileSystem;
use phocommander_ops::queue::JobQueue;
use phocommander_ops::undo::UndoLedger;
use phocommander_ui::commands::Command;
use phocommander_ui::panel::PanelSide;
use phocommander_ui::surface::TransferRequest;
use phocommander_ui::surface::WorkSurface;
use phocommander_viewer::policy::ReadPolicy;
use phocommander_viewer::reader::ViewerReader;
use std::sync::Arc;

#[test]
fn the_undo_takes_the_copy_back() {
    // arrange — two panels and a file that is copied from the one into the
    // other.
    let fs = FakeFileSystem::new();
    fs.write_file(std::path::Path::new("/photos/a.txt"), b"hallo")
        .unwrap_or_else(|error| panic!("the file should be writable: {error:?}"));
    let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
    let mut surface = WorkSurface::new("/photos".into(), "/backup".into());
    let viewer = Box::new(ViewerPreviewSource::new(ViewerReader::new(
        ReadPolicy::strict(),
        10_485_760,
    )));
    let mut undo = UndoLedger::new();

    let transfer = TransferRequest::new(
        PanelSide::Left,
        PanelSide::Right,
        vec!["a.txt".to_string().into()],
    )
    .unwrap_or_else(|error| panic!("the transfer should be valid: {error:?}"));
    let mut adapter = Adapter {
        queue: &queue,
        fs: &fs,
        viewer: viewer.as_ref(),
        ways: Ways::SAFE,
        rules: CopyPolicy::strict(),
        undo: &mut undo,
    };

    // act — the file is copied, and the copy is taken back.
    let kopiert = phocommander_desktop::execute::submit(
        &[Command::Copy { transfer }],
        &mut surface,
        &mut adapter,
    );
    let zurueckgenommen =
        phocommander_desktop::execute::submit(&[Command::Undo], &mut surface, &mut adapter);

    // assert — the copy became a job, and the undo took the copy back.
    assert_eq!(kopiert, 1);
    assert_eq!(zurueckgenommen, 1);
    assert!(
        !fs.exists(std::path::Path::new("/backup/a.txt")),
        "the copy should be taken back"
    );
    assert!(
        fs.exists(std::path::Path::new("/photos/a.txt")),
        "the source should stay"
    );
}
