//! The tests of the module above.
//!
// WHY: the file-size rule of the project keeps every source file below
// 400 lines. The tests of a module live in the directory of the module,
// so that the implementation and its proof can grow independently.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

/// Creates an empty directory below the temporary directory of the system.
fn temporary_directory(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("phocommander-local-{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap_or_else(|error| {
        panic!("the temporary directory {base:?} could not be created: {error}");
    });
    base
}

#[test]
fn a_listing_names_the_entries_of_the_directory() {
    // arrange
    let directory = temporary_directory("listing");
    fs::write(directory.join("b.txt"), b"b").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    fs::write(directory.join("a.txt"), b"a").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    fs::create_dir(directory.join("photos")).unwrap_or_else(|error| {
        panic!("the directory could not be created: {error}");
    });
    let source = LocalSource::new();

    // act
    let entries = source.list(&directory);

    // assert
    let entries = match entries {
        Ok(entries) => entries,
        Err(error) => panic!("the directory could not be listed: {error}"),
    };
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].name, "a.txt");
    assert_eq!(entries[0].kind, EntryKind::File);
    assert_eq!(entries[0].size, Some(1));
    assert_eq!(entries[1].name, "b.txt");
    assert_eq!(entries[2].name, "photos");
    assert_eq!(entries[2].kind, EntryKind::Directory);
    assert_eq!(entries[2].size, None);
}

#[test]
fn a_listing_of_a_missing_directory_is_an_error() {
    // arrange
    let source = LocalSource::new();

    // act
    let result = source.list(Path::new("/does/not/exist/at/all"));

    // assert
    assert!(result.is_err());
}

#[test]
#[cfg(unix)]
fn a_link_is_listed_as_a_link_and_without_a_size() {
    // arrange
    let directory = temporary_directory("link-listing");
    let target = directory.join("target.txt");
    fs::write(&target, b"payload").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    std::os::unix::fs::symlink(&target, directory.join("link.txt")).unwrap_or_else(|error| {
        panic!("the link could not be created: {error}");
    });
    let source = LocalSource::new();

    // act
    let entries = source.list(&directory);

    // assert
    let entries = match entries {
        Ok(entries) => entries,
        Err(error) => panic!("the directory could not be listed: {error}"),
    };
    let link = entries
        .iter()
        .find(|entry| entry.name == "link.txt")
        .unwrap_or_else(|| panic!("the link is missing from {entries:?}"));
    assert_eq!(link.kind, EntryKind::Symlink);
    assert_eq!(link.size, None);
}

#[test]
fn a_file_is_copied_byte_by_byte() {
    // arrange
    let directory = temporary_directory("copy");
    fs::write(directory.join("a.txt"), b"payload").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    let source = LocalSource::new();

    // act
    let result = source.copy(&directory.join("a.txt"), &directory.join("b.txt"));

    // assert
    assert!(result.is_ok());
    let content = fs::read(directory.join("b.txt")).unwrap_or_else(|error| {
        panic!("the file could not be read: {error}");
    });
    assert_eq!(content, b"payload");
}

#[test]
#[cfg(unix)]
fn a_link_is_copied_as_a_link() {
    // arrange
    let directory = temporary_directory("copy-link");
    let target = directory.join("target.txt");
    fs::write(&target, b"payload").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    std::os::unix::fs::symlink(&target, directory.join("link.txt")).unwrap_or_else(|error| {
        panic!("the link could not be created: {error}");
    });
    let source = LocalSource::new();

    // act
    let result = source.copy(&directory.join("link.txt"), &directory.join("copy.txt"));

    // assert
    assert!(result.is_ok());
    let copied = fs::symlink_metadata(directory.join("copy.txt")).unwrap_or_else(|error| {
        panic!("the copy could not be inspected: {error}");
    });
    assert!(copied.file_type().is_symlink());
}

#[test]
#[cfg(unix)]
fn a_link_is_dereferenced_when_the_rules_allow_it() {
    // WHY: the rule of the specification is that a link is copied as a link.
    // The switch that allows the copy to dereference it is named after the
    // deviation from that rule.
    // arrange
    let directory = temporary_directory("copy-link-followed");
    let target = directory.join("target.txt");
    fs::write(&target, b"payload").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    std::os::unix::fs::symlink(&target, directory.join("link.txt")).unwrap_or_else(|error| {
        panic!("the link could not be created: {error}");
    });
    let source = LocalSource::new().with_follow_symlinks(true);

    // act
    let result = source.copy(&directory.join("link.txt"), &directory.join("copy.txt"));

    // assert
    assert!(result.is_ok());
    let copied = fs::symlink_metadata(directory.join("copy.txt")).unwrap_or_else(|error| {
        panic!("the copy could not be inspected: {error}");
    });
    assert!(!copied.file_type().is_symlink());
    let content = fs::read(directory.join("copy.txt")).unwrap_or_else(|error| {
        panic!("the copy could not be read: {error}");
    });
    assert_eq!(content, b"payload");
}

#[test]
fn a_source_follows_no_link_by_default() {
    // act
    let source = LocalSource::new();

    // assert
    assert!(!source.follows_symlinks());
}

#[test]
fn a_file_is_moved_and_renamed() {
    // arrange
    let directory = temporary_directory("move");
    fs::write(directory.join("a.txt"), b"payload").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    let source = LocalSource::new();

    // act
    let moved = source.move_entry(&directory.join("a.txt"), &directory.join("b.txt"));
    let renamed = source.rename(&directory.join("b.txt"), &directory.join("c.txt"));

    // assert
    assert!(moved.is_ok());
    assert!(renamed.is_ok());
    assert!(directory.join("c.txt").exists());
    assert!(!directory.join("a.txt").exists());
}

#[test]
fn a_file_is_removed_and_a_directory_is_created() {
    // arrange
    let directory = temporary_directory("remove");
    fs::write(directory.join("a.txt"), b"payload").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    let source = LocalSource::new();

    // act
    let removed = source.remove(&directory.join("a.txt"));
    let created = source.create_directory(&directory.join("photos/2026"));

    // assert
    assert!(removed.is_ok());
    assert!(created.is_ok());
    assert!(!directory.join("a.txt").exists());
    assert!(directory.join("photos/2026").is_dir());
}

#[test]
fn a_rooted_source_refuses_a_path_that_leaves_the_root() {
    // arrange
    let directory = temporary_directory("rooted");
    let source = LocalSource::rooted(&directory);

    // act
    let result = source.resolve(Path::new("../escape.txt"));

    // assert
    assert!(result.is_err());
}

#[test]
fn a_rooted_source_resolves_a_path_inside_the_root() {
    // arrange
    let directory = temporary_directory("rooted-inside");
    let source = LocalSource::rooted(&directory);

    // act
    let path = source.resolve(Path::new("photos/sunset.jpg"));

    // assert
    let path = match path {
        Ok(path) => path,
        Err(error) => panic!("the path should be accepted: {error}"),
    };
    assert!(path.starts_with(&directory), "{path:?}");
}

#[test]
fn the_checksum_of_a_file_is_computed_from_its_content() {
    // arrange
    let directory = temporary_directory("checksum");
    let file = directory.join("a.txt");
    fs::write(&file, b"123456789").unwrap_or_else(|error| {
        panic!("the file could not be written: {error}");
    });
    let source = LocalSource::new();

    // act
    let checksum = source.checksum(&file);

    // assert
    let checksum = match checksum {
        Ok(checksum) => checksum,
        Err(error) => panic!("the checksum could not be computed: {error}"),
    };
    assert_eq!(checksum, 0xCBF4_3926);
}
