//! The documents of a source that hands out handles instead of paths.
//!
//! WHAT: [{Document}] — one entry of a tree of documents that a source
//!       addresses by an identifier instead of by a path — and the
//!       [{DocumentTree}], the collection of them with the rules that a
//!       panel needs.
//! WHY: on Android the storage of the system is addressed by
//!       `content://`-URIs, and the identifier of a document says nothing
//!       about where the document lies. A path is a name of a place in a
//!       tree of directories; an identifier of a document is a name that
//!       only the source understands. The core therefore knows the
//!       documents and not the URIs, and the source translates them at its
//!       boundary — the same way the source of a released directory on
//!       iPadOS keeps its bookmark to itself (ADR-0002).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::document::{Document, DocumentId, DocumentKind, DocumentTree};
//!
//! let mut tree = DocumentTree::new();
//! tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
//! tree.insert(Document::file(DocumentId::new("root/1"), "a.jpg").with_size(2048));
//! assert_eq!(tree.children_of(&DocumentId::new("root")).len(), 1);
//! assert_eq!(tree.name_of(&DocumentId::new("root/1")), Some("a.jpg"));
//! ```

use std::collections::BTreeMap;

/// The identifier of a document.
///
/// WHY: the identifier is the only thing that every source can deliver
///       about the place of a document. It is opaque: the core compares
///       identifiers and never interprets them, because the meaning of an
///       identifier belongs to the source that handed it out.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(String);

impl DocumentId {
    /// Builds an identifier.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::DocumentId;
    ///
    /// let id = DocumentId::new("root/1");
    /// assert_eq!(id.as_str(), "root/1");
    /// ```
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The kind of a document.
///
/// WHY: the kind decides what the interface shows and what the operations
///       may do with it. A folder can be entered, an archive can be opened
///       and a picture can be shown — and a document of a kind that the
///       product does not know is still a file that can be copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentKind {
    /// A directory that holds other documents.
    Folder,

    /// A file of any kind.
    File,

    /// An archive that can be opened like a directory.
    Archive,

    /// A picture that the viewer can show.
    Picture,

    /// A sound file.
    Audio,

    /// A video file.
    Video,

    /// A text document that the viewer can show.
    Text,
}

impl DocumentKind {
    /// Whether the document can be entered like a directory.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::DocumentKind;
    ///
    /// assert!(DocumentKind::Folder.can_be_entered());
    /// assert!(!DocumentKind::File.can_be_entered());
    /// ```
    pub const fn can_be_entered(&self) -> bool {
        matches!(self, Self::Folder | Self::Archive)
    }
}

/// The permission that the user granted for a document.
///
/// WHY: the permission is part of the document and not part of the source,
///       because the permission belongs to the tree the user released. A
///       document that the user may read but not write is shown like every
///       other document, and the operations that would write to it are
///       refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// The document can be read.
    Read,

    /// The document can be read and written.
    ReadWrite,
}

impl Permission {
    /// Whether the document can be written to.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::Permission;
    ///
    /// assert!(Permission::ReadWrite.allows_writing());
    /// assert!(!Permission::Read.allows_writing());
    /// ```
    pub const fn allows_writing(&self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

/// One document of a tree.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::document::{Document, DocumentId};
///
/// let document = Document::file(DocumentId::new("root/1"), "a.txt");
/// assert_eq!(document.name(), "a.txt");
/// assert!(!document.is_folder());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    id: DocumentId,
    name: String,
    kind: DocumentKind,
    parent: Option<DocumentId>,
    size: Option<u64>,
    permission: Permission,
}

impl Document {
    /// Builds a document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId, DocumentKind};
    ///
    /// let document = Document::new(
    ///     DocumentId::new("root/1"),
    ///     "a.txt",
    ///     DocumentKind::File,
    ///     Some(DocumentId::new("root")),
    /// );
    /// assert_eq!(document.name(), "a.txt");
    /// ```
    pub fn new(
        id: DocumentId,
        name: impl Into<String>,
        kind: DocumentKind,
        parent: Option<DocumentId>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            parent,
            size: None,
            permission: Permission::ReadWrite,
        }
    }

    /// Builds a folder.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId};
    ///
    /// assert!(Document::folder(DocumentId::new("root"), "Photos").is_folder());
    /// ```
    pub fn folder(id: DocumentId, name: impl Into<String>) -> Self {
        Self::new(id, name, DocumentKind::Folder, None)
    }

    /// Builds a file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId};
    ///
    /// assert!(!Document::file(DocumentId::new("root/1"), "a.txt").is_folder());
    /// ```
    pub fn file(id: DocumentId, name: impl Into<String>) -> Self {
        Self::new(id, name, DocumentKind::File, None)
    }

    /// Sets the size of the document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId};
    ///
    /// let document = Document::file(DocumentId::new("root/1"), "a.txt").with_size(12);
    /// assert_eq!(document.size(), Some(12));
    /// ```
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the folder that holds the document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId};
    ///
    /// let document = Document::file(DocumentId::new("root/1"), "a.txt")
    ///     .in_folder(DocumentId::new("root"));
    /// assert_eq!(document.parent(), Some(&DocumentId::new("root")));
    /// ```
    pub fn in_folder(mut self, parent: DocumentId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Sets the permission of the document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId, Permission};
    ///
    /// let document = Document::file(DocumentId::new("root/1"), "a.txt")
    ///     .with_permission(Permission::Read);
    /// assert!(!document.permission().allows_writing());
    /// ```
    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permission = permission;
        self
    }

    /// The identifier of the document.
    pub const fn id(&self) -> &DocumentId {
        &self.id
    }

    /// The name of the document.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The kind of the document.
    pub const fn kind(&self) -> DocumentKind {
        self.kind
    }

    /// The folder that holds the document, if it is in one.
    pub fn parent(&self) -> Option<&DocumentId> {
        self.parent.as_ref()
    }

    /// The size of the document, if the source knows it.
    pub const fn size(&self) -> Option<u64> {
        self.size
    }

    /// The permission that the user granted for the document.
    pub const fn permission(&self) -> Permission {
        self.permission
    }

    /// Whether the document can be entered like a directory.
    pub const fn is_folder(&self) -> bool {
        self.kind.can_be_entered()
    }
}

/// The documents of a source, and the rules that a panel needs.
///
/// WHY: the panel of a source shows the children of one folder. The tree
///       knows the documents, and the rules — the children of a folder, the
///       way to a folder, the name of a document — stand in one place, so
///       that the interface and the operations ask the same questions
///       (ADR-0002).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::document::{Document, DocumentId, DocumentTree};
///
/// let mut tree = DocumentTree::new();
/// tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
/// tree.insert(
///     Document::file(DocumentId::new("root/1"), "a.jpg")
///         .in_folder(DocumentId::new("root")),
/// );
/// assert_eq!(tree.children_of(&DocumentId::new("root")).len(), 1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DocumentTree {
    documents: BTreeMap<DocumentId, Document>,
}

impl DocumentTree {
    /// The tree without documents.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::DocumentTree;
    ///
    /// assert!(DocumentTree::new().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts a document into the tree.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId, DocumentTree};
    ///
    /// let mut tree = DocumentTree::new();
    /// tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
    /// assert_eq!(tree.len(), 1);
    /// ```
    pub fn insert(&mut self, document: Document) {
        self.documents.insert(document.id().clone(), document);
    }

    /// The number of the documents.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::DocumentTree;
    ///
    /// assert_eq!(DocumentTree::new().len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Whether the tree holds no document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::DocumentTree;
    ///
    /// assert!(DocumentTree::new().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// The document with the given identifier.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId, DocumentTree};
    ///
    /// let mut tree = DocumentTree::new();
    /// tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
    /// assert!(tree.get(&DocumentId::new("root")).is_some());
    /// ```
    pub fn get(&self, id: &DocumentId) -> Option<&Document> {
        self.documents.get(id)
    }

    /// The name of the document with the given identifier.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId, DocumentTree};
    ///
    /// let mut tree = DocumentTree::new();
    /// tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
    /// assert_eq!(tree.name_of(&DocumentId::new("root")), Some("Photos"));
    /// ```
    pub fn name_of(&self, id: &DocumentId) -> Option<&str> {
        self.get(id).map(Document::name)
    }

    /// The children of a folder, in the order of their names.
    ///
    /// WHY: the list of a panel is sorted by name, and the folders come
    ///       first — the same order on every source, so that the user does
    ///       not have to learn a new order on every device.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId, DocumentTree};
    ///
    /// let mut tree = DocumentTree::new();
    /// tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
    /// tree.insert(
    ///     Document::file(DocumentId::new("root/1"), "a.jpg")
    ///         .in_folder(DocumentId::new("root")),
    /// );
    /// tree.insert(
    ///     Document::folder(DocumentId::new("root/2"), "Zoo")
    ///         .in_folder(DocumentId::new("root")),
    /// );
    /// let children = tree.children_of(&DocumentId::new("root"));
    /// assert_eq!(children.len(), 2);
    /// assert_eq!(children[0].name(), "Zoo");
    /// ```
    pub fn children_of(&self, folder: &DocumentId) -> Vec<&Document> {
        let mut children: Vec<&Document> = self
            .documents
            .values()
            .filter(|document| document.parent() == Some(folder))
            .collect();
        children.sort_by(|left, right| {
            right
                .is_folder()
                .cmp(&left.is_folder())
                .then_with(|| left.name().cmp(right.name()))
        });
        children
    }

    /// The way from a document up to the root of the tree.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::document::{Document, DocumentId, DocumentTree};
    ///
    /// let mut tree = DocumentTree::new();
    /// tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
    /// tree.insert(
    ///     Document::folder(DocumentId::new("root/1"), "2024")
    ///         .in_folder(DocumentId::new("root")),
    /// );
    /// tree.insert(
    ///     Document::file(DocumentId::new("root/1/2"), "a.jpg")
    ///         .in_folder(DocumentId::new("root/1")),
    /// );
    /// let way = tree.way_to_root(&DocumentId::new("root/1/2"));
    /// assert_eq!(way.len(), 2);
    /// ```
    pub fn way_to_root(&self, id: &DocumentId) -> Vec<&Document> {
        let mut way = Vec::new();
        let mut current = id.clone();
        // WHY: the tree of a source can be deep, and a loop in it would
        //       never end. The walk stops after a hundred steps, and a
        //       source that builds such a tree has a fault of its own.
        let mut steps = 0;
        while let Some(document) = self.get(&current) {
            way.push(document);
            steps += 1;
            if steps > 100 {
                break;
            }
            match document.parent() {
                Some(parent) => current = parent.clone(),
                None => break,
            }
        }
        way
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    /// A tree with a folder and two files.
    fn tree() -> DocumentTree {
        let mut tree = DocumentTree::new();
        tree.insert(Document::folder(DocumentId::new("root"), "Photos"));
        tree.insert(
            Document::file(DocumentId::new("root/1"), "a.jpg")
                .in_folder(DocumentId::new("root"))
                .with_size(2048),
        );
        tree.insert(
            Document::folder(DocumentId::new("root/2"), "Zoo").in_folder(DocumentId::new("root")),
        );
        tree
    }

    #[test]
    fn the_children_of_a_folder_are_the_documents_of_it() {
        // arrange
        let tree = tree();
        let root = DocumentId::new("root");

        // act
        let children = tree.children_of(&root);

        // assert
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn the_folders_come_before_the_files() {
        // arrange
        let tree = tree();
        let root = DocumentId::new("root");

        // act
        let children = tree.children_of(&root);

        // assert — the order is the same on every source.
        assert_eq!(children[0].name(), "Zoo");
        assert_eq!(children[1].name(), "a.jpg");
    }

    #[test]
    fn the_way_to_the_root_names_every_step() {
        // arrange
        let mut tree = tree();
        tree.insert(
            Document::file(DocumentId::new("root/1/2"), "b.jpg")
                .in_folder(DocumentId::new("root/1")),
        );

        // act
        let way = tree.way_to_root(&DocumentId::new("root/1/2"));

        // assert
        assert_eq!(way.len(), 3);
        assert_eq!(way[0].name(), "b.jpg");
        assert_eq!(way[2].name(), "Photos");
    }

    #[test]
    fn the_permission_says_whether_the_document_can_be_written_to() {
        // arrange
        let document =
            Document::file(DocumentId::new("root/1"), "a.txt").with_permission(Permission::Read);

        // act, assert
        assert!(!document.permission().allows_writing());
    }

    #[test]
    fn a_loop_in_the_tree_ends_the_walk() {
        // arrange — two documents that are the parent of each other.
        let mut tree = DocumentTree::new();
        tree.insert(Document::folder(DocumentId::new("a"), "A").in_folder(DocumentId::new("b")));
        tree.insert(Document::folder(DocumentId::new("b"), "B").in_folder(DocumentId::new("a")));

        // act
        let way = tree.way_to_root(&DocumentId::new("a"));

        // assert — the walk ends and does not spin.
        assert!(way.len() <= 101, "the walk should end: {}", way.len());
    }
}
