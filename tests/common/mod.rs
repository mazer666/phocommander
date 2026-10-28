//! Shared helpers of the integration tests of Phocommander.
//!
//! WHAT: the helpers every integration test of the project needs — a temporary
//!       directory that is removed again, the creation of a file with a given
//!       content, the creation of a directory tree, and the inspection of a
//!       directory for the negative tests.
//! WHY:  a test that leaves its files behind is a test that can fail for the
//!       wrong reason — the next run finds the leftovers of the previous one.
//!       A test that writes its hostile archive into the repository is worse:
//!       a binary file in the history is an attack sample that nobody asked
//!       for. Therefore every temporary artefact of a test lives in a
//!       directory that is created at run time and deleted afterwards.
//!       The helpers panic when they cannot do their job. That is deliberate:
//!       this crate exists for tests only, it is never part of the
//!       application, and a helper that fails loudly with the path it could
//!       not create gives a better error message than a silent empty result.
//! EXAMPLE:
//! ```no_run
//! use phocommander_testkit::TempTree;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let tree = TempTree::new("demo")?;
//! let file = tree.write_file("notes.txt", b"hello");
//! assert!(file.exists());
//! // The directory is removed when `tree` is dropped.
//! # Ok(())
//! # }
//! ```

pub mod counting;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Counter that makes the name of a temporary directory unique.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A temporary directory that removes itself when it is dropped.
///
/// WHY: the cleanup happens in `Drop`, so it also happens when a test panics.
/// A test that leaves its directory behind would make the next run of the same
/// test depend on the previous one.
#[derive(Debug)]
pub struct TempTree {
    root: PathBuf,
}

impl TempTree {
    /// Creates a temporary directory whose name starts with `label`.
    ///
    /// WHY: the label makes the directory recognisable in a file list while a
    /// test is running. The uniqueness comes from the process id, the current
    /// time and a counter, because two tests may run at the same time and the
    /// operating system must not hand the same directory to both.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_testkit::TempTree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = TempTree::new("phocommander")?;
    /// assert!(tree.path().is_dir());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(label: &str) -> Result<Self, std::io::Error> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            "phocommander-{label}-{}-{nanos}-{unique}",
            std::process::id()
        );
        let root = std::env::temp_dir().join(name);
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Returns the path of the temporary directory.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_testkit::TempTree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = TempTree::new("demo")?;
    /// assert!(tree.path().starts_with(std::env::temp_dir()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Writes a file with the given content and returns its path.
    ///
    /// WHY: the parent directories are created as well, so a test can describe
    /// a tree and not repeat the directory creation for every file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_testkit::TempTree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = TempTree::new("demo")?;
    /// let file = tree.write_file("a/b.txt", b"content");
    /// assert!(file.exists());
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_file(&self, relative: &str, content: &[u8]) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut file = match fs::File::create(&path) {
            Ok(file) => file,
            Err(error) => panic!("the file {path:?} could not be created: {error}"),
        };
        if let Err(error) = file.write_all(content) {
            panic!("the file {path:?} could not be written: {error}");
        }
        if let Err(error) = file.sync_all() {
            panic!("the file {path:?} could not be flushed: {error}");
        }
        path
    }

    /// Creates a directory tree from a list of relative paths.
    ///
    /// WHY: a test that describes its tree as a list of paths reads like the
    /// scenario it describes and needs no knowledge of the file system.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_testkit::TempTree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = TempTree::new("demo")?;
    /// tree.create_tree(&["a/b/c.txt", "a/d.txt"]);
    /// assert!(tree.path().join("a/b/c.txt").exists());
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_tree(&self, relative_paths: &[&str]) {
        for relative in relative_paths {
            self.write_file(relative, b"");
        }
    }

    /// Returns the names of the entries of a directory, sorted.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_testkit::TempTree;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tree = TempTree::new("demo")?;
    /// tree.create_tree(&["b.txt", "a.txt"]);
    /// assert_eq!(tree.entry_names("."), vec!["a.txt".to_string(), "b.txt".to_string()]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn entry_names(&self, relative: &str) -> Vec<String> {
        directory_names(&self.root.join(relative))
    }
}

impl Drop for TempTree {
    /// Removes the temporary directory and everything in it.
    ///
    /// WHY: the removal is best effort. A file that cannot be removed — a
    /// file that is still open, for example — must not turn a green test into
    /// a red one; the leftover in the temporary directory is the lesser
    /// evil.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Returns the root directory of the repository.
///
/// WHY: a test binary is started with the package directory as its working
///       directory, not the root of the repository. A test that wants to read
///       `config/defaults.toml` therefore has to find the repository root
///       first. The path is taken from the compile time constant of cargo, so
///       the test does not depend on the working directory at all.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_testkit::repository_root;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert!(repository_root().join("Cargo.toml").exists());
/// # Ok(())
/// # }
/// ```
pub fn repository_root() -> PathBuf {
    let package_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match package_directory.parent() {
        Some(parent) => parent.to_path_buf(),
        None => package_directory,
    }
}

/// Returns the names of the entries of a directory, sorted.
///
/// WHY: the negative tests need to look at what an archive would have written
/// and compare it with what the destination directory really contains. The
/// comparison needs a stable order on both sides.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_testkit::{directory_names, TempTree};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let tree = TempTree::new("demo")?;
/// tree.write_file("a.txt", b"x");
/// assert_eq!(directory_names(tree.path()), vec!["a.txt".to_string()]);
/// # Ok(())
/// # }
/// ```
pub fn directory_names(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = match fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect(),
        Err(error) => panic!("the directory {directory:?} could not be read: {error}"),
    };
    names.sort();
    names
}

/// Returns the number of files below a directory, recursively.
///
/// WHY: the negative tests assert that nothing was written outside of the
/// destination. Counting the files below the directory that surrounds the
/// destination is the cheapest way to notice an escaped file.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_testkit::{count_files, TempTree};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let tree = TempTree::new("demo")?;
/// tree.write_file("a/b.txt", b"x");
/// assert_eq!(count_files(tree.path()), 1);
/// # Ok(())
/// # }
/// ```
pub fn count_files(directory: &Path) -> usize {
    let mut count = 0;
    let mut pending = vec![directory.to_path_buf()];
    while let Some(current) = pending.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            if file_type.is_dir() {
                pending.push(path);
            } else {
                count += 1;
            }
        }
    }
    count
}
