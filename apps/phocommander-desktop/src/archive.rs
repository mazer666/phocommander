//! The browsing of the archives of the desktop build.
//!
//! WHAT: the adapter that lists the entries of an archive
//! ([`list_entries`]).
//! WHY: the viewer shows the contents of an archive, and the archive crate
//! owns the formats. The adapter is the place where the two meet: it opens
//! the container, walks over its entries and hands the names to the surface
//! (FR-54).
//! The container decides itself whether it can be opened. A file that is no
//! archive is refused by the reader of the container, and the refusal is the
//! answer — the adapter asks the container and not the extension of the
//! file.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::archive::list_entries;
//!
//! let eintraege = list_entries(std::path::Path::new("/photos/archiv.zip"));
//! // WHY: a file that is no archive gives no entries and no error.
//! let _ = eintraege;
//! ```

use std::path::Path;

use phocommander_archive::entry::ArchiveEntry;
use phocommander_archive::formats::{EntryVisitor, VisitDecision, open_archive};

/// One entry of an archive, as the surface shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveItem {
    /// The name of the entry, as the container carries it.
    pub name: String,

    /// Whether the entry is a directory.
    pub is_directory: bool,

    /// The number of the bytes the entry announces.
    pub size: u64,
}

/// The visitor that collects the entries of an archive.
#[derive(Debug, Default)]
struct Collector {
    /// The entries that were seen.
    items: Vec<ArchiveItem>,
}

impl EntryVisitor for Collector {
    fn visit(
        &mut self,
        entry: &ArchiveEntry,
        _content: Option<&mut dyn std::io::Read>,
    ) -> Result<VisitDecision, phocommander_archive::error::ArchiveError> {
        self.items.push(ArchiveItem {
            name: entry.name.clone(),
            is_directory: matches!(entry.kind, phocommander_archive::EntryKind::Directory),
            size: entry.size,
        });
        Ok(VisitDecision::Continue)
    }
}

/// The entries of the archive at the given path.
///
/// WHY: a file that is no archive gives no entries and no error. The
///       surface asks every file whether it is an archive, and a text file
///       is no archive — the answer of the container is the answer.
pub fn list_entries(path: &Path) -> Vec<ArchiveItem> {
    let Ok(mut reader) = open_archive(path) else {
        return Vec::new();
    };
    let mut collector = Collector::default();
    if reader.for_each_entry(&mut collector).is_err() {
        return Vec::new();
    }
    collector.items
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_entries_of_an_archive_are_listed() {
        // arrange
        // WHY: the archive that the test builds is a real ZIP file on the
        //       disk. The reader of the container reads it, and the
        //       adapter hands the names to the surface.
        use phocommander_archive::formats::ArchiveWriter;
        use phocommander_archive::formats::zip::ZipArchiveWriter;

        let pfad = std::env::temp_dir().join("phocommander-archive-entries.zip");
        let _ = std::fs::remove_file(&pfad);
        {
            let mut writer = ZipArchiveWriter::create(&pfad)
                .unwrap_or_else(|error| panic!("the archive should have been created: {error}"));
            writer
                .add_file("notes.txt", 5, &mut std::io::Cursor::new(b"hallo".to_vec()))
                .unwrap_or_else(|error| panic!("the file should have been added: {error}"));
            writer
                .add_directory("ordner")
                .unwrap_or_else(|error| panic!("the directory should have been added: {error}"));
            Box::new(writer)
                .finish()
                .unwrap_or_else(|error| panic!("the archive should have been closed: {error}"));
        }

        // act
        let eintraege = list_entries(&pfad);

        // assert
        assert_eq!(eintraege.len(), 2, "{eintraege:?}");
        // WHY: the ZIP format writes the name of a directory with a slash
        //       at the end — that is the sign of the format that it is a
        //       directory.
        let namen: Vec<&str> = eintraege
            .iter()
            .map(|eintrag| eintrag.name.as_str())
            .collect();
        assert!(namen.contains(&"notes.txt"), "{namen:?}");
        assert!(namen.contains(&"ordner/"), "{namen:?}");
        let ordner = eintraege
            .iter()
            .find(|eintrag| eintrag.name.trim_end_matches('/') == "ordner")
            .unwrap_or_else(|| panic!("the directory should be listed"));
        assert!(ordner.is_directory);
        let datei = eintraege
            .iter()
            .find(|eintrag| eintrag.name == "notes.txt")
            .unwrap_or_else(|| panic!("the file should be listed"));
        assert_eq!(datei.size, 5);
        assert!(!datei.is_directory);

        let _ = std::fs::remove_file(&pfad);
    }

    #[test]
    fn a_file_that_is_no_archive_gives_no_entries() {
        // act, assert
        // WHY: the surface asks every file whether it is an archive, and a
        //       text file is no archive.
        assert!(list_entries(Path::new("/gibt-es-nicht.zip")).is_empty());
    }
}
