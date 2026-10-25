//! The tests of the module above.
//!
// WHY: the file-size rule of the project keeps every source file below
// 400 lines. The tests of a module live in the directory of the module,
// so that the implementation and its proof can grow independently.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

#[test]
fn io_error_names_the_path_and_keeps_the_source() {
    // arrange
    let path = Path::new("/tmp/missing.txt");
    let source = io::Error::new(io::ErrorKind::NotFound, "no such file");

    // act
    let error = Error::io(path, source);

    // assert
    let message = error.to_string();
    assert!(message.contains("/tmp/missing.txt"), "{message}");
    assert!(message.contains("no such file"), "{message}");
}

#[test]
fn too_large_error_names_size_and_limit() {
    // arrange
    let path = Path::new("movie.mkv");

    // act
    let error = Error::too_large(path, 20_000_000, 10_485_760);

    // assert
    let message = error.to_string();
    assert!(message.contains("movie.mkv"), "{message}");
    assert!(message.contains("20000000"), "{message}");
    assert!(message.contains("10485760"), "{message}");
}

#[test]
fn io_error_reports_the_original_error_as_source() {
    // arrange
    let source = io::Error::new(io::ErrorKind::PermissionDenied, "denied");

    // act
    let error = Error::io(Path::new("/root"), source);

    // assert
    let source = std::error::Error::source(&error);
    let kind = source
        .and_then(|source| source.downcast_ref::<io::Error>())
        .map(io::Error::kind);
    assert_eq!(kind, Some(io::ErrorKind::PermissionDenied));
}

#[test]
fn a_refused_access_carries_the_permission_code() {
    // arrange
    let source = io::Error::new(io::ErrorKind::PermissionDenied, "denied");

    // act
    let error = Error::write(Path::new("/root/x"), source);

    // assert
    assert_eq!(error.code(), ErrorCode::IoPermission);
    assert_eq!(error.code().as_str(), "io.permission");
    assert_eq!(error.code().number(), 1003);
}

#[test]
fn a_failed_read_and_a_failed_write_carry_different_codes() {
    // arrange
    let read = Error::io(
        Path::new("a.txt"),
        io::Error::new(io::ErrorKind::NotFound, "missing"),
    );
    let write = Error::write(Path::new("a.txt"), io::Error::other("broken"));

    // act, assert
    assert_eq!(read.code(), ErrorCode::IoRead);
    assert_eq!(write.code(), ErrorCode::IoWrite);
}

#[test]
fn a_missing_configuration_directory_carries_its_own_code() {
    // act
    let error = Error::NoConfigDirectory;

    // assert
    assert_eq!(error.code(), ErrorCode::PlatformNoConfigDirectory);
    assert_eq!(error.code().as_str(), "platform.no_config_directory");
}

#[test]
fn a_file_that_is_too_large_carries_the_limit_code() {
    // act
    let error = Error::too_large(Path::new("movie.mkv"), 20_000_000, 10);

    // assert
    assert_eq!(error.code(), ErrorCode::LimitTooLarge);
}

#[test]
fn a_refused_path_keeps_its_code() {
    // act
    let error = Error::from(PathError::Traversal {
        path: PathBuf::from("../escape.txt"),
    });

    // assert
    assert_eq!(error.code(), ErrorCode::PathTraversal);
}

#[test]
fn every_code_has_a_name_and_a_number_that_no_other_code_uses() {
    // arrange
    let codes = [
        ErrorCode::IoRead,
        ErrorCode::IoWrite,
        ErrorCode::IoPermission,
        ErrorCode::PathAbsolute,
        ErrorCode::PathTraversal,
        ErrorCode::PathReserved,
        ErrorCode::LimitTooLarge,
        ErrorCode::LimitEntries,
        ErrorCode::PlatformNoConfigDirectory,
    ];

    // act
    let names: Vec<&str> = codes.iter().map(ErrorCode::as_str).collect();
    let numbers: Vec<u16> = codes.iter().map(ErrorCode::number).collect();

    // assert
    for (index, name) in names.iter().enumerate() {
        assert!(!name.is_empty());
        assert_eq!(
            names.iter().filter(|other| *other == name).count(),
            1,
            "{name} is used twice"
        );
        assert_eq!(
            numbers
                .iter()
                .filter(|other| *other == &numbers[index])
                .count(),
            1,
            "{} is used twice",
            numbers[index]
        );
    }
}
