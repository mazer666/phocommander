//! What the user may do with a document of a source that holds permissions.
//!
//! WHAT: [{DocumentPolicy}] — the rules that say which operation a document
//!       allows, and why it does not allow it.
//! WHY: a document that the user released for reading may be read and not
//!       written. A program that hides the difference shows the user a
//!       button that does nothing, and a program that refuses without a
//!       reason leaves the user with the impression that the program is
//!       broken. The rules therefore carry the reason with them, and the
//!       reason is a key of the catalogue — the interface says it in the
//!       language of the user (FR-05, FR-121).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::document::{Document, DocumentId, Permission};
//! use phocommander_core::policy::{DocumentOperation, DocumentPolicy};
//!
//! let document = Document::file(DocumentId::new("root/1"), "a.txt")
//!     .with_permission(Permission::Read);
//! let policy = DocumentPolicy::of(&document);
//! assert!(policy.allows(DocumentOperation::Copy));
//! assert!(!policy.allows(DocumentOperation::Delete));
//! assert!(policy.reason(DocumentOperation::Delete).is_some());
//! ```

use crate::document::{Document, Permission};

/// The operations that a document of a source can refuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentOperation {
    /// The document is read.
    Read,

    /// The document is copied.
    Copy,

    /// The document is moved.
    Move,

    /// The document is deleted.
    Delete,

    /// The document is renamed.
    Rename,

    /// A new document is written into the folder of the document.
    CreateInside,
}

impl DocumentOperation {
    /// Every operation, in a stable order.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::policy::DocumentOperation;
    ///
    /// assert_eq!(DocumentOperation::all().len(), 6);
    /// ```
    pub const fn all() -> [DocumentOperation; 6] {
        [
            Self::Read,
            Self::Copy,
            Self::Move,
            Self::Delete,
            Self::Rename,
            Self::CreateInside,
        ]
    }

    /// Whether the operation changes the document itself.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::policy::DocumentOperation;
    ///
    /// assert!(DocumentOperation::Delete.changes_the_document());
    /// assert!(!DocumentOperation::Copy.changes_the_document());
    /// ```
    pub const fn changes_the_document(&self) -> bool {
        matches!(self, Self::Move | Self::Delete | Self::Rename)
    }
}

/// The rules that say what a document allows.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::document::{Document, DocumentId, Permission};
/// use phocommander_core::policy::{DocumentOperation, DocumentPolicy};
///
/// let document = Document::file(DocumentId::new("root/1"), "a.txt")
///     .with_permission(Permission::Read);
/// let policy = DocumentPolicy::of(&document);
/// assert!(!policy.allows(DocumentOperation::Rename));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentPolicy {
    permission: Permission,
}

impl DocumentPolicy {
    /// The rules of a document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId};
    /// use phocommander_core::policy::DocumentPolicy;
    ///
    /// let document = Document::file(DocumentId::new("root/1"), "a.txt");
    /// assert!(DocumentPolicy::of(&document).allows_writing());
    /// ```
    pub const fn of(document: &Document) -> Self {
        Self {
            permission: document.permission(),
        }
    }

    /// The rules of a permission.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::Permission;
    /// use phocommander_core::policy::DocumentPolicy;
    ///
    /// let policy = DocumentPolicy::with_permission(Permission::Read);
    /// assert!(!policy.allows_writing());
    /// ```
    pub const fn with_permission(permission: Permission) -> Self {
        Self { permission }
    }

    /// Whether the document may be written to at all.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::Permission;
    /// use phocommander_core::policy::DocumentPolicy;
    ///
    /// assert!(DocumentPolicy::with_permission(Permission::ReadWrite).allows_writing());
    /// ```
    pub const fn allows_writing(&self) -> bool {
        self.permission.allows_writing()
    }

    /// Whether the operation is allowed.
    ///
    /// WHY: reading and copying are always allowed — a document that the
    ///       user may read may be read and copied. Everything that writes
    ///       needs the permission to write.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::Permission;
    /// use phocommander_core::policy::{DocumentOperation, DocumentPolicy};
    ///
    /// let policy = DocumentPolicy::with_permission(Permission::Read);
    /// assert!(policy.allows(DocumentOperation::Read));
    /// assert!(policy.allows(DocumentOperation::Copy));
    /// assert!(!policy.allows(DocumentOperation::Move));
    /// ```
    pub const fn allows(&self, operation: DocumentOperation) -> bool {
        match operation {
            DocumentOperation::Read | DocumentOperation::Copy => true,
            DocumentOperation::Move
            | DocumentOperation::Delete
            | DocumentOperation::Rename
            | DocumentOperation::CreateInside => self.permission.allows_writing(),
        }
    }

    /// The key of the catalogue that says why the operation is refused.
    ///
    /// WHY: the reason is a key and not a sentence, because the sentence
    ///       belongs to the catalogue and to the language of the user
    ///       (FR-05). A caller that does not know the key can show it as
    ///       it is — the key names the reason.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::Permission;
    /// use phocommander_core::policy::{DocumentOperation, DocumentPolicy};
    ///
    /// let policy = DocumentPolicy::with_permission(Permission::Read);
    /// assert_eq!(
    ///     policy.reason(DocumentOperation::Delete),
    ///     Some("policy-read-only")
    /// );
    /// assert_eq!(policy.reason(DocumentOperation::Read), None);
    /// ```
    pub const fn reason(&self, operation: DocumentOperation) -> Option<&'static str> {
        if self.allows(operation) {
            None
        } else {
            Some("policy-read-only")
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn a_document_that_may_be_written_allows_everything() {
        // arrange
        let policy = DocumentPolicy::with_permission(Permission::ReadWrite);

        // act, assert
        for operation in DocumentOperation::all() {
            assert!(policy.allows(operation), "{operation:?} should be allowed");
            assert_eq!(policy.reason(operation), None);
        }
    }

    #[test]
    fn a_document_that_may_only_be_read_refuses_the_operations_that_write() {
        // arrange
        let policy = DocumentPolicy::with_permission(Permission::Read);

        // act, assert
        assert!(policy.allows(DocumentOperation::Read));
        assert!(policy.allows(DocumentOperation::Copy));
        for operation in [
            DocumentOperation::Move,
            DocumentOperation::Delete,
            DocumentOperation::Rename,
            DocumentOperation::CreateInside,
        ] {
            assert!(!policy.allows(operation), "{operation:?} should be refused");
            assert_eq!(policy.reason(operation), Some("policy-read-only"));
        }
    }

    #[test]
    fn the_policy_of_a_document_is_the_policy_of_its_permission() {
        // arrange
        let document = Document::file(crate::document::DocumentId::new("root/1"), "a.txt")
            .with_permission(Permission::Read);

        // act
        let policy = DocumentPolicy::of(&document);

        // assert
        assert!(!policy.allows_writing());
        assert!(!policy.allows(DocumentOperation::Delete));
    }
}
