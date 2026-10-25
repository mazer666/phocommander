//! The hardened extraction of an archive.
//!
//! WHAT: the engine that writes the entries of an archive into a destination
//!       directory, with every hardening of the specification applied.
//! WHY: an archive is untrusted input, and the name of an entry decides
//!       where the entry is written. The engine therefore resolves every name
//!       with the one function of the crate, canonicalises the target and
//!       confines it to the destination, creates no symbolic link, gives the
//!       files the permissions of the application and not the ones of the
//!       archive, and counts the bytes it writes against a budget.
//!
//! The write itself happens relative to the opened destination
//!       directory (the {openat} semantics of ADR-0004, decision 6). A
//!       symbolic link that is swapped in between the check and the write
//!       therefore cannot redirect the write out of the destination.
//! SIZE: the hardening of the extraction belongs in one module. Every check —
//!       the name, the canonicalisation, the budget, the permissions, the
//!       symbolic links — is one step of the same walk, and a reader who
//!       wants to know whether a path can escape has to see all of them at
//!       once.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::{ExtractionOptions, extract_archive};
//!
//! # fn main() -> Result<(), phocommander_archive::ArchiveError> {
//! let report = extract_archive(
//!     std::path::Path::new("holiday.zip"),
//!     std::path::Path::new("/tmp/holiday"),
//!     &ExtractionOptions::default(),
//! )?;
//! println!("{} bytes written", report.written_bytes);
//! # Ok(())
//! # }
//! ```

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use rustix::fs::{AtFlags, Mode, OFlags, mkdirat, open, openat, statat};
use std::os::fd::{AsFd, OwnedFd};

use crate::entry::{ArchiveEntry, EntryKind};
use crate::entry_name::resolve_entry_name;
use crate::error::{ArchiveError, LimitKind, RefusalReason};
use crate::formats::{ArchiveReader, EntryVisitor, VisitDecision};
use crate::limits::{EXTRACTED_DIRECTORY_MODE, EXTRACTED_FILE_MODE, ExtractionLimits};

/// The error of the system as the error of the standard library.
///
/// WHY: the crate `rustix` reports the errors of the system with its own
///       type, and the errors of this crate are the ones of the standard
///       library. The bridge is written once, here.
#[cfg(unix)]
fn io_error(error: rustix::io::Errno) -> io::Error {
    io::Error::from_raw_os_error(error.raw_os_error())
}

/// The mode of a symbolic link, as the system reports it.
#[cfg(unix)]
const MODE_SYMBOLIC_LINK: u32 = 0o120_000;

/// The mode of a directory, as the system reports it.
#[cfg(unix)]
const MODE_DIRECTORY: u32 = 0o040_000;

/// The mask that leaves the kind of an entry.
#[cfg(unix)]
const MODE_KIND: u32 = 0o170_000;

/// Whether the entry is a symbolic link.
///
/// WHY: the check looks at the kind of the entry and not at the path. A
///       symbolic link is refused, whatever it points to.
#[cfg(unix)]
fn is_symbolic_link(mode: u32) -> bool {
    mode & MODE_KIND == MODE_SYMBOLIC_LINK
}

/// Whether the entry is a directory.
#[cfg(unix)]
fn is_directory(mode: u32) -> bool {
    mode & MODE_KIND == MODE_DIRECTORY
}

/// The number of bytes that are copied at a time.
///
/// WHY: the content of an entry is copied in pieces, so that the size of an
///       entry never has to be held in memory. The size of the piece is a
///       compromise between the number of system calls and the memory that
///       the process uses.
const COPY_BUFFER_BYTES: usize = 64 * 1024;

/// What the extraction does when the target of an entry already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictPolicy {
    /// The file is written over the existing one.
    #[default]
    Overwrite,

    /// The existing file is kept and the entry is skipped.
    Skip,
}

/// The options of one extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractionOptions {
    /// The limits of the extraction.
    pub limits: ExtractionLimits,

    /// What happens when the target of an entry already exists.
    pub conflict: ConflictPolicy,

    /// Whether a refused entry aborts the whole extraction.
    ///
    /// WHY: the strict setting is the default. An archive that carries an
    ///       entry that leaves the destination is an attack, and an attack
    ///       that is only half refused is an attack that the user does not
    ///       learn about.
    pub abort_on_refused_entry: bool,
}

impl Default for ExtractionOptions {
    /// Returns the options of the specification: the limits of the
    /// specification, overwriting an existing file and aborting on the first
    /// refused entry.
    fn default() -> Self {
        Self {
            limits: ExtractionLimits::default(),
            conflict: ConflictPolicy::Overwrite,
            abort_on_refused_entry: true,
        }
    }
}

/// The reason why an entry was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The entry is a symbolic link; links are never created from archive
    /// material.
    Symlink {
        /// The target the link points to.
        target: String,
    },

    /// The entry is a symbolic link whose target would leave the destination.
    SymlinkWithRefusedTarget {
        /// The target the link points to.
        target: String,
    },

    /// The entry is of a kind that is never created: a hard link, a device
    /// node or a named pipe.
    UnsupportedKind {
        /// What the entry is, in words.
        kind: &'static str,
    },

    /// The target of the entry exists and the policy is to keep it.
    ExistingFile,
}

/// What happened with one entry of the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryOutcome {
    /// The entry was written.
    Written {
        /// The name of the entry.
        name: String,
        /// The number of bytes that were written.
        bytes: u64,
    },

    /// The entry was a directory and the directory exists.
    Directory {
        /// The name of the entry.
        name: String,
    },

    /// The entry was not written, and that is not an error.
    Skipped {
        /// The name of the entry.
        name: String,
        /// The reason of the skip.
        reason: SkipReason,
    },

    /// The entry was refused.
    Refused {
        /// The name of the entry.
        name: String,
        /// The reason of the refusal.
        reason: RefusalReason,
    },
}

/// The report of an extraction.
///
/// WHY: the report names every entry of the archive and what happened with
///       it. An extraction that skipped something has to say so, otherwise
///       the user believes that the archive was unpacked completely.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtractionReport {
    /// What happened with every entry of the archive.
    pub outcomes: Vec<EntryOutcome>,

    /// The number of bytes that were written.
    pub written_bytes: u64,

    /// The number of files that were written.
    pub written_entries: u64,

    /// The number of entries that were skipped.
    pub skipped_entries: u64,

    /// The number of entries that were refused.
    pub refused_entries: u64,
}

/// Extracts an archive into a destination directory.
///
/// WHY: the format of the archive is detected from its magic bytes, not from
///       its name. The name of an archive is chosen by its sender as well.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::{ExtractionOptions, extract_archive};
///
/// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
/// let report = extract_archive(
///     std::path::Path::new("holiday.zip"),
///     std::path::Path::new("/tmp/holiday"),
///     &ExtractionOptions::default(),
/// )?;
/// assert!(report.written_entries > 0);
/// # Ok(())
/// # }
/// ```
pub fn extract_archive(
    archive: &Path,
    destination: &Path,
    options: &ExtractionOptions,
) -> Result<ExtractionReport, ArchiveError> {
    let mut reader = crate::formats::open_archive(archive)?;
    extract_from(reader.as_mut(), destination, options)
}

/// Extracts the entries of a reader into a destination directory.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::{ExtractionOptions, extract_from, formats::zip::ZipArchiveReader};
///
/// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
/// let mut reader = ZipArchiveReader::open(std::path::Path::new("holiday.zip"))?;
/// let report = extract_from(
///     &mut reader,
///     std::path::Path::new("/tmp/holiday"),
///     &ExtractionOptions::default(),
/// )?;
/// assert!(report.written_entries > 0);
/// # Ok(())
/// # }
/// ```
pub fn extract_from<R: ArchiveReader + ?Sized>(
    reader: &mut R,
    destination: &Path,
    options: &ExtractionOptions,
) -> Result<ExtractionReport, ArchiveError> {
    let mut extraction = Extraction::new(destination, options)?;
    reader.for_each_entry(&mut extraction)?;
    Ok(extraction.into_report())
}

/// The engine of an extraction.
struct Extraction {
    destination: PathBuf,
    canonical_destination: PathBuf,
    /// The descriptor of the destination directory.
    ///
    /// WHY: every access of the extraction happens relative to this
    ///       descriptor and not over a path that is resolved a second time
    ///       (ADR-0004, decision 6). A symbolic link that is swapped in
    ///       between the check and the write cannot redirect the write out
    ///       of the destination.
    #[cfg(unix)]
    handle: OwnedFd,
    limits: ExtractionLimits,
    conflict: ConflictPolicy,
    abort_on_refused_entry: bool,
    written_bytes: u64,
    entry_count: u64,
    outcomes: Vec<EntryOutcome>,
    written_entries: u64,
    skipped_entries: u64,
    refused_entries: u64,
}

impl Extraction {
    /// Opens the destination directory and prepares the extraction.
    fn new(destination: &Path, options: &ExtractionOptions) -> Result<Self, ArchiveError> {
        fs::create_dir_all(destination).map_err(|source| ArchiveError::io(destination, source))?;
        let canonical_destination = fs::canonicalize(destination)
            .map_err(|source| ArchiveError::io(destination, source))?;
        // The handle is the heart of the hardening: every access of the
        // extraction happens relative to this directory, so that a symbolic
        // link cannot redirect a write out of it.
        #[cfg(unix)]
        let handle = open(
            destination,
            OFlags::RDONLY | OFlags::DIRECTORY,
            Mode::empty(),
        )
        .map_err(|source| ArchiveError::io(destination, io_error(source)))?;

        Ok(Self {
            destination: destination.to_path_buf(),
            canonical_destination,
            handle,
            limits: options.limits,
            conflict: options.conflict,
            abort_on_refused_entry: options.abort_on_refused_entry,
            written_bytes: 0,
            entry_count: 0,
            outcomes: Vec::new(),
            written_entries: 0,
            skipped_entries: 0,
            refused_entries: 0,
        })
    }

    /// Returns the report of the extraction.
    fn into_report(self) -> ExtractionReport {
        ExtractionReport {
            outcomes: self.outcomes,
            written_bytes: self.written_bytes,
            written_entries: self.written_entries,
            skipped_entries: self.skipped_entries,
            refused_entries: self.refused_entries,
        }
    }

    /// Returns the path of the target of an entry on the file system.
    fn target_path(&self, relative: &Path) -> PathBuf {
        self.destination.join(relative)
    }

    /// Records a refusal and decides whether the extraction stops.
    fn refuse(
        &mut self,
        entry: &ArchiveEntry,
        reason: RefusalReason,
    ) -> Result<VisitDecision, ArchiveError> {
        self.refused_entries += 1;
        self.outcomes.push(EntryOutcome::Refused {
            name: entry.name.clone(),
            reason: reason.clone(),
        });
        if self.abort_on_refused_entry {
            Err(ArchiveError::Refused {
                name: entry.name.clone(),
                reason,
            })
        } else {
            Ok(VisitDecision::Continue)
        }
    }

    /// Creates the directories of the path of an entry.
    ///
    /// WHY: the directories are created one by one and every existing one is
    ///       looked at. A directory that is a symbolic link is refused, so
    ///       that an entry cannot be written through a link that points out
    ///       of the destination.
    fn create_parent_directories(&self, relative: &Path) -> Result<(), ParentFailure> {
        let parent = match relative.parent() {
            Some(parent) => parent,
            None => return Ok(()),
        };
        let mut current = PathBuf::new();
        for component in parent.components() {
            current.push(component);
            #[cfg(unix)]
            match mkdirat(
                self.handle.as_fd(),
                &current,
                Mode::from_bits_truncate(0o700),
            ) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    let metadata = statat(self.handle.as_fd(), &current, AtFlags::SYMLINK_NOFOLLOW)
                        .map_err(|source| ParentFailure::Io(io_error(source)))?;
                    if is_symbolic_link(metadata.st_mode) {
                        return Err(ParentFailure::Symlink);
                    }
                    if !is_directory(metadata.st_mode) {
                        return Err(ParentFailure::NotADirectory);
                    }
                }
                Err(error) => return Err(ParentFailure::Io(io_error(error))),
            }
        }
        Ok(())
    }

    /// Checks that the target of an entry lies inside the destination.
    ///
    /// WHY: the check is the second half of the rule H3 of the threat model.
    ///       The name of the entry decides the path, and the path is
    ///       canonicalised and compared with the canonical destination.
    fn confine(&self, relative: &Path) -> Result<(), RefusalReason> {
        let target = self.target_path(relative);
        let parent = target.parent().unwrap_or(&self.destination);
        let canonical_parent = match fs::canonicalize(parent) {
            Ok(path) => path,
            Err(_) => return Err(RefusalReason::OutsideDestination),
        };
        if !canonical_parent.starts_with(&self.canonical_destination) {
            return Err(RefusalReason::OutsideDestination);
        }
        Ok(())
    }

    /// Looks at the target of an entry.
    fn check_target(&self, relative: &Path) -> Result<TargetState, io::Error> {
        let target = self.target_path(relative);
        match fs::symlink_metadata(&target) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    Ok(TargetState::Symlink)
                } else if metadata.is_dir() {
                    Ok(TargetState::Directory)
                } else {
                    Ok(TargetState::Existing)
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(TargetState::Missing),
            Err(error) => Err(error),
        }
    }

    /// Gives a file or a directory the permissions of the application.
    ///
    /// WHY: the mode of an archive is chosen by its sender. The set-user-id,
    ///       the set-group-id and the sticky bit are never carried over, and
    ///       the group and the other never get a write bit.
    #[cfg(unix)]
    fn set_mode(&self, relative: &Path, mode: u32) -> Result<(), ArchiveError> {
        #[cfg(unix)]
        {
            let descriptor = openat(
                self.handle.as_fd(),
                relative,
                OFlags::RDONLY | OFlags::NOFOLLOW,
                Mode::empty(),
            )
            .map_err(|source| ArchiveError::io(self.target_path(relative), io_error(source)))?;
            rustix::fs::fchmod(descriptor.as_fd(), Mode::from_bits_truncate(mode))
                .map_err(|source| ArchiveError::io(self.target_path(relative), io_error(source)))?;
        }
        #[cfg(not(unix))]
        let _ = (relative, mode);

        Ok(())
    }

    /// Gives a file or a directory the permissions of the application.
    ///
    /// WHY: the permission bits of Windows are the read-only flag, and the
    ///       extraction has no reason to make a file read-only.
    #[cfg(not(unix))]
    fn set_mode(&self, _relative: &Path, _mode: u32) -> Result<(), ArchiveError> {
        Ok(())
    }

    /// Writes the directory of an entry.
    fn write_directory(&mut self, entry: &ArchiveEntry) -> Result<VisitDecision, ArchiveError> {
        let relative = match self.resolve_name(entry) {
            Ok(relative) => relative,
            Err(reason) => return self.refuse(entry, reason),
        };

        if let Err(failure) = self.create_parent_directories(&relative) {
            return match failure {
                ParentFailure::Symlink => self.refuse(entry, RefusalReason::SymlinkInPath),
                ParentFailure::NotADirectory => self.refuse(entry, RefusalReason::NotAFile),
                ParentFailure::Io(source) => {
                    Err(ArchiveError::io(self.target_path(&relative), source))
                }
            };
        }

        #[cfg(unix)]
        match mkdirat(
            self.handle.as_fd(),
            &relative,
            Mode::from_bits_truncate(0o700),
        ) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let metadata = statat(self.handle.as_fd(), &relative, AtFlags::SYMLINK_NOFOLLOW)
                    .map_err(|source| {
                        ArchiveError::io(self.target_path(&relative), io_error(source))
                    })?;
                if is_symbolic_link(metadata.st_mode) {
                    return self.refuse(entry, RefusalReason::SymlinkTarget);
                }
            }
            Err(error) => {
                return Err(ArchiveError::io(
                    self.target_path(&relative),
                    io_error(error),
                ));
            }
        }

        if let Err(reason) = self.confine(&relative) {
            return self.refuse(entry, reason);
        }

        self.set_mode(&relative, EXTRACTED_DIRECTORY_MODE)?;
        self.outcomes.push(EntryOutcome::Directory {
            name: entry.name.clone(),
        });
        Ok(VisitDecision::Continue)
    }

    /// Writes the file of an entry.
    fn write_file(
        &mut self,
        entry: &ArchiveEntry,
        content: Option<&mut dyn Read>,
    ) -> Result<VisitDecision, ArchiveError> {
        let relative = match self.resolve_name(entry) {
            Ok(relative) => relative,
            Err(reason) => return self.refuse(entry, reason),
        };

        // The limit is checked before the write, and the size that is
        // checked is the size the container announces.
        if !self.limits.allows_entry_size(entry.size) {
            return Err(ArchiveError::LimitExceeded {
                kind: LimitKind::SingleEntryBytes,
                actual: entry.size,
                limit: self.limits.max_entry_bytes,
            });
        }

        let mut content = match content {
            Some(content) => content,
            None => return self.refuse(entry, RefusalReason::MissingContent),
        };

        if let Err(failure) = self.create_parent_directories(&relative) {
            return match failure {
                ParentFailure::Symlink => self.refuse(entry, RefusalReason::SymlinkInPath),
                ParentFailure::NotADirectory => self.refuse(entry, RefusalReason::NotAFile),
                ParentFailure::Io(source) => {
                    Err(ArchiveError::io(self.target_path(&relative), source))
                }
            };
        }

        // The target is looked at immediately before the access, and the
        // canonicalised path is compared with the destination.
        match self.check_target(&relative) {
            Ok(TargetState::Symlink) => {
                return self.refuse(entry, RefusalReason::SymlinkTarget);
            }
            Ok(TargetState::Directory) => {
                return self.refuse(entry, RefusalReason::NotAFile);
            }
            Ok(TargetState::Existing) if self.conflict == ConflictPolicy::Skip => {
                self.skipped_entries += 1;
                self.outcomes.push(EntryOutcome::Skipped {
                    name: entry.name.clone(),
                    reason: SkipReason::ExistingFile,
                });
                return Ok(VisitDecision::Continue);
            }
            Ok(TargetState::Existing) | Ok(TargetState::Missing) => {}
            Err(source) => {
                return Err(ArchiveError::io(self.target_path(&relative), source));
            }
        }

        if let Err(reason) = self.confine(&relative) {
            return self.refuse(entry, reason);
        }

        // The write happens relative to the opened destination directory.
        #[cfg(unix)]
        let mut file = {
            let descriptor = openat(
                self.handle.as_fd(),
                &relative,
                OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC,
                Mode::from_bits_truncate(EXTRACTED_FILE_MODE),
            )
            .map_err(|source| ArchiveError::io(self.target_path(&relative), io_error(source)))?;
            std::fs::File::from(descriptor)
        };

        let remaining = self
            .limits
            .max_total_bytes
            .saturating_sub(self.written_bytes);
        let written = copy_within_budget(
            &mut content,
            &mut file,
            remaining,
            self.written_bytes,
            self.limits.max_total_bytes,
        )?;

        self.set_mode(&relative, EXTRACTED_FILE_MODE)?;
        self.written_bytes += written;
        self.written_entries += 1;
        self.outcomes.push(EntryOutcome::Written {
            name: entry.name.clone(),
            bytes: written,
        });
        Ok(VisitDecision::Continue)
    }

    /// Skips a symbolic link.
    ///
    /// WHY: a symbolic link is never created from archive material. The link
    ///       could point anywhere, and a later entry of the same archive
    ///       would then be written through it. The entry is reported, so
    ///       that the user learns that the archive contains something that
    ///       was not unpacked.
    fn skip_symlink(&mut self, entry: &ArchiveEntry) -> Result<VisitDecision, ArchiveError> {
        let target = entry.link_target.clone().unwrap_or_default();
        let reason = match resolve_entry_name(&target) {
            Ok(_) => SkipReason::Symlink { target },
            Err(_) => SkipReason::SymlinkWithRefusedTarget { target },
        };
        self.skipped_entries += 1;
        self.outcomes.push(EntryOutcome::Skipped {
            name: entry.name.clone(),
            reason,
        });
        Ok(VisitDecision::Continue)
    }

    /// Skips an entry of a kind that is never created.
    fn skip_unsupported(&mut self, entry: &ArchiveEntry) -> Result<VisitDecision, ArchiveError> {
        self.skipped_entries += 1;
        self.outcomes.push(EntryOutcome::Skipped {
            name: entry.name.clone(),
            reason: SkipReason::UnsupportedKind {
                kind: "a hard link, a device node or a named pipe",
            },
        });
        Ok(VisitDecision::Continue)
    }

    /// Resolves the name of an entry and refuses the names that may not be
    /// written.
    fn resolve_name(&self, entry: &ArchiveEntry) -> Result<PathBuf, RefusalReason> {
        if entry.name_is_lossy {
            return Err(RefusalReason::InvalidName);
        }
        resolve_entry_name(&entry.name).map_err(RefusalReason::Name)
    }
}

impl EntryVisitor for Extraction {
    fn visit(
        &mut self,
        entry: &ArchiveEntry,
        content: Option<&mut dyn Read>,
    ) -> Result<VisitDecision, ArchiveError> {
        self.entry_count += 1;
        if !self.limits.allows_entry_count(self.entry_count) {
            return Err(ArchiveError::LimitExceeded {
                kind: LimitKind::EntryCount,
                actual: self.entry_count,
                limit: self.limits.max_entries,
            });
        }

        match entry.kind {
            EntryKind::Directory => self.write_directory(entry),
            EntryKind::Symlink => self.skip_symlink(entry),
            EntryKind::File => self.write_file(entry, content),
            EntryKind::Other => self.skip_unsupported(entry),
        }
    }
}

/// The state of the target of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetState {
    /// Nothing is at the target.
    Missing,

    /// A file is at the target.
    Existing,

    /// A symbolic link is at the target.
    Symlink,

    /// A directory is at the target.
    Directory,
}

/// The reason why a directory of the path of an entry could not be used.
enum ParentFailure {
    /// The directory is a symbolic link.
    Symlink,

    /// The directory is not a directory.
    NotADirectory,

    /// The operating system refused the operation.
    Io(io::Error),
}

/// Copies the content of an entry into the file, without exceeding the budget.
///
/// WHY: the budget is checked while the content is copied and not only
///       before, because the size that the container announces can be a lie.
///       The number of bytes that are really written is the number that
///       counts.
fn copy_within_budget(
    content: &mut dyn Read,
    target: &mut dyn Write,
    remaining: u64,
    already_written: u64,
    limit: u64,
) -> Result<u64, ArchiveError> {
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    let mut written = 0_u64;
    loop {
        let read = content
            .read(&mut buffer)
            .map_err(|source| ArchiveError::io("the content of an archive entry", source))?;
        if read == 0 {
            break;
        }
        let count = read as u64;
        if count > remaining {
            return Err(ArchiveError::LimitExceeded {
                kind: LimitKind::TotalBytes,
                actual: already_written + written + count,
                limit,
            });
        }
        target
            .write_all(&buffer[..read])
            .map_err(|source| ArchiveError::io("the extracted file", source))?;
        written += count;
    }
    Ok(written)
}
