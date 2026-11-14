//! The rules of the Android application, in the Rust core.
//!
//! WHAT: the functions that the Java or Kotlin shell calls, and the rules
//!       they carry — the order of the documents, the operations that a
//!       document allows, and the reason for a refusal.
//! WHY: Android is the target system with the most divergent access rules:
//!       the Storage Access Framework hands out directories that the user
//!       released, and every document carries the permission that came with
//!       it. The rules belong in the core, because they are the rules of
//!       the product and not of the platform — and the core is where they
//!       can be tested without an emulator.
//!
//! BOUNDARY: this library is the bridge between the Rust core and the Java
//!       or Kotlin shell. It owns the entry points and the translation of
//!       the documents, and nothing else; the file access itself lives in
//!       the platform adapter of the shell.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_bridge::rules::Listing;
//!
//! let listing = Listing::of(&[
//!     ("root", "Photos", true, false),
//!     ("root/1", "a.jpg", false, false),
//! ]);
//! assert_eq!(listing.names().first().map(String::as_str), Some("Photos"));
//! ```

use phocommander_core::document::{Document, DocumentId, DocumentKind, DocumentTree, Permission};
use phocommander_core::policy::{DocumentOperation, DocumentPolicy};

/// One document as the shell hands it over.
///
/// WHY: the shell cannot hand over a Rust structure over JNI. It hands over
///       the four values that every document has — the identifier, the name,
///       the kind and the permission — and the core builds the tree from
///       them. The translation stays on one side of the bridge, and the
///       other side speaks only in strings and numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRecord {
    /// The identifier of the document.
    pub id: String,

    /// The name of the document.
    pub name: String,

    /// Whether the document is a folder.
    pub is_folder: bool,

    /// Whether the user may write to the document.
    pub may_write: bool,
}

impl DocumentRecord {
    /// Builds a record.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_bridge::rules::DocumentRecord;
    ///
    /// let record = DocumentRecord::new("root/1", "a.jpg", false, false);
    /// assert_eq!(record.name, "a.jpg");
    /// ```
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        is_folder: bool,
        may_write: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            is_folder,
            may_write,
        }
    }
}

/// The listing of a folder, in the order of the panel.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_bridge::rules::Listing;
///
/// let listing = Listing::of(&[("root", "Photos", true, true)]);
/// assert_eq!(listing.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    entries: Vec<ListingEntry>,
}

impl Listing {
    /// Builds the listing of the children of a folder.
    ///
    /// WHY: the shell hands over the documents of one folder, and the core
    ///       decides the order — the folders first, then the files, each
    ///       group by name. The order is the same on every platform, so
    ///       that the user does not learn a new one on every device.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_bridge::rules::Listing;
    ///
    /// let listing = Listing::of(&[
    ///     ("root/2", "Zoo", true, true),
    ///     ("root/1", "a.jpg", false, false),
    /// ]);
    /// assert_eq!(listing.names().first().map(String::as_str), Some("Zoo"));
    /// ```
    pub fn of(records: &[(impl AsRef<str>, impl AsRef<str>, bool, bool)]) -> Self {
        let mut tree = DocumentTree::new();
        for (id, name, is_folder, may_write) in records {
            let id = DocumentId::new(id.as_ref());
            let mut document = if *is_folder {
                Document::folder(id.clone(), name.as_ref())
            } else {
                Document::file(id.clone(), name.as_ref())
            };
            document = document.with_permission(if *may_write {
                Permission::ReadWrite
            } else {
                Permission::Read
            });
            tree.insert(document);
        }
        // WHY: the documents of a folder arrive without their parent, and
        //       the children of the folder are the documents whose
        //       identifier begins with the identifier of the folder.
        let mut entries: Vec<ListingEntry> = records
            .iter()
            .map(|(id, name, is_folder, may_write)| {
                let kind = if *is_folder {
                    DocumentKind::Folder
                } else {
                    DocumentKind::File
                };
                let permission = if *may_write {
                    Permission::ReadWrite
                } else {
                    Permission::Read
                };
                let document =
                    Document::new(DocumentId::new(id.as_ref()), name.as_ref(), kind, None)
                        .with_permission(permission);
                let policy = DocumentPolicy::of(&document);
                ListingEntry {
                    id: id.as_ref().to_string(),
                    name: name.as_ref().to_string(),
                    is_folder: *is_folder,
                    may_write: *may_write,
                    may_delete: policy.allows(DocumentOperation::Delete),
                    may_rename: policy.allows(DocumentOperation::Rename),
                }
            })
            .collect();
        entries.sort_by(|left, right| {
            right
                .is_folder
                .cmp(&left.is_folder)
                .then_with(|| left.name.cmp(&right.name))
        });
        Self { entries }
    }

    /// The rows of the listing.
    pub fn entries(&self) -> &[ListingEntry] {
        &self.entries
    }

    /// The names of the rows, in the order of the panel.
    pub fn names(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| entry.name.clone())
            .collect()
    }

    /// The number of the rows.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the listing holds no row.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// One row of a listing.
///
/// WHY: the row carries what the panel needs and nothing else. The
///       operations that the document refuses are named here, so that the
///       shell does not have to ask again for every button of every row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListingEntry {
    /// The identifier of the document.
    pub id: String,

    /// The name of the document.
    pub name: String,

    /// Whether the document is a folder.
    pub is_folder: bool,

    /// Whether the user may write to the document.
    pub may_write: bool,

    /// Whether the document may be deleted.
    pub may_delete: bool,

    /// Whether the document may be renamed.
    pub may_rename: bool,
}

/// The key of the catalogue that says why an operation is refused.
///
/// WHY: the shell shows the reason in the language of the user, and the
///       key belongs to the core, because the core knows when a document
///       refuses (FR-05).
pub const READ_ONLY_REASON: &str = "policy-read-only";

/// The reason why a document refuses an operation.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_bridge::rules::READ_ONLY_REASON;
///
/// assert_eq!(READ_ONLY_REASON, "policy-read-only");
/// ```
pub const fn reason_of(may_write: bool) -> Option<&'static str> {
    if may_write {
        None
    } else {
        Some(READ_ONLY_REASON)
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_folders_come_before_the_files() {
        // arrange
        let records = [
            ("root/1", "a.jpg", false, true),
            ("root/2", "Zoo", true, true),
            ("root/3", "b.jpg", false, true),
        ];

        // act
        let listing = Listing::of(&records);

        // assert
        assert_eq!(listing.names(), vec!["Zoo", "a.jpg", "b.jpg"]);
    }

    #[test]
    fn a_document_that_may_not_be_written_refuses_the_operations_that_write() {
        // arrange
        let records = [("root/1", "a.jpg", false, false)];

        // act
        let listing = Listing::of(&records);

        // assert
        let entry = &listing.entries()[0];
        assert!(!entry.may_write);
        assert!(!entry.may_delete);
        assert!(!entry.may_rename);
    }

    #[test]
    fn a_document_that_may_be_written_allows_the_operations_that_write() {
        // arrange
        let records = [("root/1", "a.jpg", false, true)];

        // act
        let listing = Listing::of(&records);

        // assert
        let entry = &listing.entries()[0];
        assert!(entry.may_write);
        assert!(entry.may_delete);
        assert!(entry.may_rename);
    }

    #[test]
    fn the_reason_is_the_key_of_the_catalogue() {
        // act, assert
        assert_eq!(reason_of(false), Some("policy-read-only"));
        assert_eq!(reason_of(true), None);
    }
}
