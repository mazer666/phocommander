//! The file operations of Phocommander.
//!
//! WHAT: the crate that owns copying, moving, renaming and deleting files
//!       ([{copy}]), the conflict behaviour of a copy ([{edit}]), the errors
//!       ([{error}]), the execution of a job ([{executor}]), the file system
//!       the operations work on ([{port}]), the progress of a job
//!       ([{progress}]), the queue that runs the jobs ([{queue}]), the
//!       comparison of two trees ([{compare}]), the plan of a synchronisation
//!       ([{sync}]), the atomic multiple rename ([{rename}]), the formation
//!       of the new names of a rename ([{template}]), the temporary file of
//!       the atomic copy ([{temporary}]) and the undo of a copy ([{undo}]).
//! WHY: every operation that can take longer than a frame belongs into the
//!       job queue, so that the user interface stays usable while a copy is
//!       running. The conflict behaviour — overwrite, skip, keep both, ask —
//!       belongs here as well, because the rule is part of the operation and
//!       not of the dialog that asks the user.
//! BOUNDARY: this crate knows the file system and nothing else. It draws
//!       nothing, asks no question and knows no platform.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::copy::{ConflictPolicy, CopyMode, CopyRequest};
//! use std::path::PathBuf;
//!
//! // The default of a copy is the mode that verifies.
//! assert_eq!(CopyMode::default(), CopyMode::Verified);
//!
//! let request = CopyRequest::new(PathBuf::from("/photos"), PathBuf::from("/backup"))
//!     .with_policy(ConflictPolicy::Never);
//! assert_eq!(request.policy, ConflictPolicy::Never);
//! assert_eq!(request.mode, CopyMode::Verified);
//! ```

pub mod compare;
pub mod copy;
pub mod datetime;
pub mod edit;
pub mod error;
pub mod executor;
pub mod fake_fs;
pub mod job;
pub mod names;
pub mod port;
pub mod progress;
pub mod queue;
pub mod rename;
pub mod series;
pub mod sync;
pub mod template;
pub mod temporary;
pub mod undo;

pub use compare::{ComparisonDepth, compare_directories, compare_trees, compare_trees_with};
pub use copy::{
    CopyMode, CopyOutcome, CopyPolicy, CopyRequest, TreeItem, collect_items, copy_one_file,
    execute_copy, free_name,
};
pub use edit::{ConflictPolicy, Edit, EditError, JobControl};
pub use error::{JobError, JobErrorCode, NameProblem, OpError};
pub use job::{Job, JobId, JobKind, JobSnapshot, JobState, Priority};
pub use port::{CopyResult, DirEntry, EntryKind, FileInfo, FileSystem};
pub use progress::{JobContext, ProgressSink, ProgressUpdate};
pub use queue::JobQueue;
pub use rename::{RenameConflict, RenameConflictKind, RenamePair, execute_renames, plan_renames};
pub use sync::{
    Approval, SyncAction, SyncPlan, SyncPlanEntry, SyncState, build_plan, build_plan_with,
    execute_sync,
};
pub use template::{
    CounterOrder, CounterRule, FormedName, Placeholder, RenameInput, RenameRule, ReplaceRule,
    SeriesRule, TemplateError, form_names, name_problems, rename_pairs,
};
pub use temporary::{TemporaryFile, temporary_path};
pub use undo::{UndoEntry, UndoLedger, UndoOutcome, UndoScope};
