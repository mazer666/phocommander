//! The file operations of Phocommander.
//!
//! WHAT: the crate that owns copying, moving, renaming and deleting files
//!       and the job queue that runs them in the background.
//! WHY: every operation that can take longer than a frame belongs into the
//!       job queue, so that the user interface stays usable while a copy is
//!       running. The conflict behaviour — overwrite, skip, keep both, ask —
//!       belongs here as well, because the rule is part of the operation and
//!       not of the dialog that asks the user.
//! BOUNDARY: this crate knows the file system and nothing else. It draws
//!       nothing, asks no question and knows no platform.
//!
//! MILESTONE: the crate is created in M0 so that the boundary exists from the
//! first day. The job queue, the operations and the conflict behaviour arrive
//! in M1, together with their unit tests.
//!
//! EXAMPLE:
//! ```no_run
//! // Milestone M1 will look like this:
//! // let job = phocommander_ops::Job::copy(source, destination);
//! // queue.enqueue(job)?;
//! ```
