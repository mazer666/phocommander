//! The directory tree of a panel.
//!
//! WHAT: one node of the tree ([`TreeNode`]), the kind of a node
//!       ([`TreeNodeKind`]) and the tree of a panel ([`DirectoryTree`]).
//! WHY: the tree shows the hierarchy around the directory that is shown and
//!       nothing else. It never walks the file system by itself: it is fed
//!       with the directories that the user has opened — the ancestors of
//!       the current path — and with the directories that lie directly below
//!       the current path (FR-14). A tree that read the whole disk on its
//!       own would cost seconds on a large volume and would show a hierarchy
//!       that the user never asked for.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::tree::DirectoryTree;
//! use std::path::{Path, PathBuf};
//!
//! let mut tree = DirectoryTree::new();
//! tree.set_root(Path::new("/home/user"), vec![PathBuf::from("/home/user/notes")]);
//! assert_eq!(tree.len(), 3);
//! ```

use std::path::{Path, PathBuf};

/// The kind of a node of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeNodeKind {
    /// A directory above the current path — one the user has opened.
    Ancestor,

    /// The directory that is shown right now.
    Current,

    /// A directory directly below the current path.
    Child,
}

/// One node of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    path: PathBuf,
    name: String,
    depth: u16,
    kind: TreeNodeKind,
}

impl TreeNode {
    /// The path of the directory of the node.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The name of the directory, without the path above it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The distance of the node from the root of the tree.
    pub fn depth(&self) -> u16 {
        self.depth
    }

    /// The kind of the node.
    pub fn kind(&self) -> TreeNodeKind {
        self.kind
    }

    /// Whether the node is the directory that is shown right now.
    pub fn is_current(&self) -> bool {
        self.kind == TreeNodeKind::Current
    }
}

/// The directory tree of a panel.
///
/// WHY: the tree holds the nodes it shows and the place of the cursor in
/// them. It is fed by the panel — the panel knows the path that is shown and
/// the entries of that path — and it never reads the file system itself.
#[derive(Debug, Default)]
pub struct DirectoryTree {
    nodes: Vec<TreeNode>,
    cursor: usize,
}

impl DirectoryTree {
    /// A tree without nodes.
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the tree has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The nodes of the tree.
    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    /// The number of the node the cursor is on.
    pub fn cursor(&self) -> usize {
        self.cursor.min(self.nodes.len().saturating_sub(1))
    }

    /// The node the cursor is on.
    pub fn cursor_node(&self) -> Option<&TreeNode> {
        self.nodes.get(self.cursor())
    }

    /// The path of the node the cursor is on.
    pub fn cursor_path(&self) -> Option<&Path> {
        self.cursor_node().map(TreeNode::path)
    }

    /// Moves the cursor by a number of nodes.
    ///
    /// WHY: the cursor of the tree never leaves the tree. A cursor that
    /// points at a node that does not exist would make the jump of the
    /// keyboard meaningless.
    pub fn move_cursor(&mut self, delta: isize) {
        if self.nodes.is_empty() {
            return;
        }
        let last = self.nodes.len() as isize - 1;
        let target = (self.cursor() as isize + delta).clamp(0, last);
        self.cursor = target as usize;
    }

    /// Puts the cursor on the node with the given path.
    ///
    /// WHY: the tree follows the path that is shown. When the user opens a
    /// directory, the cursor of the tree is on that directory.
    pub fn focus_path(&mut self, path: &Path) {
        if let Some(index) = self.nodes.iter().position(|node| node.path() == path) {
            self.cursor = index;
        }
    }

    /// Rebuilds the tree around the given path.
    ///
    /// WHY: the tree shows the ancestors of the current path — the
    /// directories that the user has opened on the way there — and the
    /// directories that lie directly below it. Nothing else is read, and
    /// nothing below the current path is read recursively (FR-14).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::panel::tree::DirectoryTree;
    /// use std::path::{Path, PathBuf};
    ///
    /// let mut tree = DirectoryTree::new();
    /// tree.set_root(Path::new("/home/user"), vec![PathBuf::from("/home/user/notes")]);
    /// assert!(tree.nodes().iter().any(|node| node.is_current()));
    /// ```
    pub fn set_root(&mut self, path: &Path, children: Vec<PathBuf>) {
        let mut nodes = Vec::new();
        let mut ancestors: Vec<PathBuf> = Vec::new();
        let mut current = Some(path.to_path_buf());

        while let Some(directory) = current {
            ancestors.push(directory.clone());
            current = directory.parent().map(Path::to_path_buf);
            if ancestors.len() > 64 {
                break;
            }
        }
        ancestors.reverse();

        let root_depth = ancestors.len().saturating_sub(1) as u16;
        for (index, ancestor) in ancestors.iter().enumerate() {
            let is_current = ancestor.as_path() == path;
            nodes.push(TreeNode {
                path: ancestor.clone(),
                name: name_of(ancestor),
                depth: index as u16,
                kind: if is_current {
                    TreeNodeKind::Current
                } else {
                    TreeNodeKind::Ancestor
                },
            });
        }

        for child in children {
            nodes.push(TreeNode {
                name: name_of(&child),
                path: child,
                depth: root_depth.saturating_add(1),
                kind: TreeNodeKind::Child,
            });
        }

        self.nodes = nodes;
        self.focus_path(path);
    }
}

/// The name of a directory, without the path above it.
///
/// WHY: the root of a volume has no parent and no name of its own, so the
/// whole path is shown there.
fn name_of(path: &Path) -> String {
    match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => path.to_string_lossy().to_string(),
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_tree_shows_the_ancestors_and_the_direct_children() {
        // arrange
        let mut tree = DirectoryTree::new();

        // act
        tree.set_root(
            Path::new("/home/user/notes"),
            vec![PathBuf::from("/home/user/notes/diary")],
        );

        // assert
        // WHY: the root of the volume is the first ancestor of every path,
        //       and the tree shows it like every other ancestor. The name of
        //       the root is the whole path, because the root has no name of
        //       its own.
        let names: Vec<&str> = tree.nodes().iter().map(TreeNode::name).collect();
        assert_eq!(names, vec!["/", "home", "user", "notes", "diary"]);
        assert_eq!(tree.nodes()[3].kind(), TreeNodeKind::Current);
        assert_eq!(tree.nodes()[4].kind(), TreeNodeKind::Child);
        assert_eq!(tree.nodes()[0].kind(), TreeNodeKind::Ancestor);
    }

    #[test]
    fn the_cursor_never_leaves_the_tree() {
        // arrange
        let mut tree = DirectoryTree::new();
        tree.set_root(Path::new("/home"), Vec::new());
        let last = tree.nodes().len() - 1;

        // act
        tree.move_cursor(-5);
        tree.move_cursor(50);

        // assert
        // WHY: the cursor is held between the first and the last node. It
        //       must not run over the first node and it must not stop behind
        //       the last one.
        assert_eq!(tree.cursor(), last);
        assert!(tree.cursor() <= last);
    }

    #[test]
    fn the_cursor_follows_the_path_that_is_shown() {
        // arrange
        let mut tree = DirectoryTree::new();
        tree.set_root(
            Path::new("/home/user"),
            vec![PathBuf::from("/home/user/notes")],
        );

        // act
        tree.focus_path(Path::new("/home/user/notes"));

        // assert
        assert_eq!(tree.cursor_path(), Some(Path::new("/home/user/notes")));
    }
}
