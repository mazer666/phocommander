//! The tests of the module above.
//!
// WHY: the file-size rule of the project keeps every source file below
// 400 lines. The tests of a module live in the directory of the module,
// so that the implementation and its proof can grow independently.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use std::fs;

/// Creates a directory below the temporary directory of the system.
fn temporary_directory(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("phocommander-path-{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap_or_else(|error| {
        panic!("the temporary directory {base:?} could not be created: {error}");
    });
    base
}

#[test]
fn a_relative_name_stays_inside_the_destination() {
    // arrange
    let base = temporary_directory("inside");
    let nested = base.join("docs");
    fs::create_dir_all(&nested).unwrap_or_else(|error| {
        panic!("the directory {nested:?} could not be created: {error}");
    });

    // act
    let target = resolve_within(
        &base,
        Path::new("docs/readme.md"),
        CaseSensitivity::Sensitive,
    );

    // assert
    let target = match target {
        Ok(target) => target,
        Err(error) => panic!("the name should be accepted: {error}"),
    };
    assert!(target.starts_with(&base), "{target:?}");
    assert_eq!(
        target.file_name().and_then(OsStr::to_str),
        Some("readme.md")
    );
}

#[test]
fn a_name_that_leaves_the_destination_is_refused() {
    // arrange
    let base = temporary_directory("escape");

    // act
    let result = resolve_within(
        &base,
        Path::new("../escape.txt"),
        CaseSensitivity::Sensitive,
    );

    // assert
    assert!(
        matches!(result, Err(PathError::Traversal { .. })),
        "{result:?}"
    );
}

#[test]
fn an_absolute_name_is_refused() {
    // arrange
    let base = temporary_directory("absolute");

    // act
    let result = resolve_within(&base, Path::new("/etc/passwd"), CaseSensitivity::Sensitive);

    // assert
    assert!(
        matches!(result, Err(PathError::Absolute { .. })),
        "{result:?}"
    );
}

#[test]
fn a_name_with_a_drive_letter_is_refused() {
    // arrange
    let base = temporary_directory("drive");

    // act
    let result = resolve_within(
        &base,
        Path::new("C:\\Windows\\evil.dll"),
        CaseSensitivity::Sensitive,
    );

    // assert
    assert!(
        matches!(result, Err(PathError::Absolute { .. })),
        "{result:?}"
    );
}

#[test]
fn a_name_that_is_reserved_on_windows_is_refused() {
    // arrange
    let base = temporary_directory("reserved");

    // act
    let result = resolve_within(&base, Path::new("NUL.txt"), CaseSensitivity::Sensitive);

    // assert
    assert!(
        matches!(result, Err(PathError::Reserved { .. })),
        "{result:?}"
    );
}

#[test]
fn a_name_with_a_forbidden_character_is_refused() {
    // arrange
    let base = temporary_directory("forbidden");

    // act
    let result = resolve_within(&base, Path::new("what?.txt"), CaseSensitivity::Sensitive);

    // assert
    assert!(
        matches!(result, Err(PathError::Reserved { .. })),
        "{result:?}"
    );
}

#[test]
#[cfg(unix)]
fn a_link_that_points_out_of_the_destination_is_refused() {
    // arrange
    let base = temporary_directory("symlink");
    let outside = temporary_directory("symlink-outside");
    let link = base.join("escape");
    std::os::unix::fs::symlink(&outside, &link).unwrap_or_else(|error| {
        panic!("the link {link:?} could not be created: {error}");
    });

    // act
    let result = resolve_within(
        &base,
        Path::new("escape/file.txt"),
        CaseSensitivity::Sensitive,
    );

    // assert
    assert!(
        matches!(result, Err(PathError::Traversal { .. })),
        "{result:?}"
    );
}

#[test]
#[cfg(unix)]
fn a_link_that_is_the_target_itself_is_refused() {
    // arrange
    let base = temporary_directory("symlink-target");
    let outside = temporary_directory("symlink-target-outside");
    let target = outside.join("file.txt");
    fs::write(&target, b"payload").unwrap_or_else(|error| {
        panic!("the file {target:?} could not be written: {error}");
    });
    let link = base.join("link.txt");
    std::os::unix::fs::symlink(&target, &link).unwrap_or_else(|error| {
        panic!("the link {link:?} could not be created: {error}");
    });

    // act
    let result = resolve_within(&base, Path::new("link.txt"), CaseSensitivity::Sensitive);

    // assert
    assert!(
        matches!(result, Err(PathError::Traversal { .. })),
        "{result:?}"
    );
}

#[test]
fn the_case_policy_decides_whether_a_path_is_inside() {
    // arrange
    let base = Path::new("/tmp/destination");

    // act
    let sensitive = is_within(
        Path::new("/TMP/destination/a.txt"),
        base,
        CaseSensitivity::Sensitive,
    );
    let insensitive = is_within(
        Path::new("/TMP/destination/a.txt"),
        base,
        CaseSensitivity::Insensitive,
    );

    // assert
    assert!(!sensitive);
    assert!(insensitive);
}

#[test]
fn a_shared_prefix_is_not_inside_the_destination() {
    // arrange
    let base = Path::new("/tmp/destination");

    // act
    let inside = is_within(
        Path::new("/tmp/destination-other/a.txt"),
        base,
        CaseSensitivity::Sensitive,
    );

    // assert
    assert!(!inside);
}

#[test]
fn two_names_that_differ_only_in_the_case_are_the_same_name_on_macos() {
    // act
    let insensitive = same_name("Report.txt", "report.txt", CaseSensitivity::Insensitive);
    let sensitive = same_name("Report.txt", "report.txt", CaseSensitivity::Sensitive);

    // assert
    assert!(insensitive);
    assert!(!sensitive);
}

#[test]
fn the_normalisation_removes_the_segments_that_carry_no_information() {
    // act
    let normalized = match normalize_relative(Path::new("./docs/../readme.md")) {
        Ok(normalized) => normalized,
        Err(error) => panic!("the path should be accepted: {error}"),
    };

    // assert
    assert_eq!(normalized.to_string_lossy(), "readme.md");
}

#[test]
fn the_normalisation_refuses_a_path_that_leaves_the_destination() {
    // act
    let result = normalize_relative(Path::new("../../escape.txt"));

    // assert
    assert!(
        matches!(result, Err(PathError::Traversal { .. })),
        "{result:?}"
    );
}

#[test]
fn the_normalisation_refuses_an_absolute_path() {
    // act
    let result = normalize_relative(Path::new("/etc/passwd"));

    // assert
    assert!(
        matches!(result, Err(PathError::Absolute { .. })),
        "{result:?}"
    );
}

#[test]
fn the_device_names_of_windows_are_recognised() {
    // act
    let reserved = ["CON", "nul.txt", "PRN", "COM1", "LPT9.log"];
    let allowed = ["concert.txt", "comics", "null"];

    // assert
    for name in reserved {
        assert!(is_reserved_device_name(name), "{name} was accepted");
    }
    for name in allowed {
        assert!(!is_reserved_device_name(name), "{name} was refused");
    }
}

#[test]
fn every_path_failure_carries_its_code() {
    // act
    let absolute = PathError::Absolute {
        path: PathBuf::from("/etc/passwd"),
    };
    let traversal = PathError::Traversal {
        path: PathBuf::from("../escape.txt"),
    };
    let reserved = PathError::Reserved {
        name: "NUL".to_string(),
    };

    // assert
    assert_eq!(absolute.code(), crate::error::ErrorCode::PathAbsolute);
    assert_eq!(traversal.code(), crate::error::ErrorCode::PathTraversal);
    assert_eq!(reserved.code(), crate::error::ErrorCode::PathReserved);
}
