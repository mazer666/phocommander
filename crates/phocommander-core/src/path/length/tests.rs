//! The tests of the module above.
//!
//! WHY: the file-size rule of the project keeps every source file below
//! 400 lines. The tests of a module live in the directory of the module,
//! so that the implementation and its proof can grow independently.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

/// Builds a path that is longer than the given number of characters.
///
/// WHY: the limit of Windows is a number of characters, and the test has to
///       name a path that is longer than it. The path is built from a name
///       that Windows accepts, so that the test does not depend on the file
///       system of the machine that runs it.
fn long_path(characters: usize) -> PathBuf {
    let mut path = String::from("C:\\");
    while path.chars().count() <= characters {
        path.push_str("long-directory-name\\");
    }
    PathBuf::from(path)
}

#[test]
fn a_path_that_is_longer_than_the_limit_of_windows_is_refused() {
    // arrange
    let path = long_path(WINDOWS_MAX_PATH_CHARACTERS);

    // act
    let result = check_path_length(&path, PathLengthLimit::windows());

    // assert
    let error = match result {
        Ok(()) => panic!("the path should be refused"),
        Err(error) => error,
    };
    assert!(
        error.amount() > WINDOWS_MAX_PATH_CHARACTERS,
        "{}",
        error.amount()
    );
    assert_eq!(error.path(), path.as_path());
    assert_eq!(error.limit().amount(), WINDOWS_MAX_PATH_CHARACTERS);
}

#[test]
fn a_path_of_the_limit_of_windows_is_accepted() {
    // arrange
    let mut text = String::from("C:\\");
    while text.chars().count() < WINDOWS_MAX_PATH_CHARACTERS {
        text.push('x');
    }
    let path = PathBuf::from(text);

    // act
    let result = check_path_length(&path, PathLengthLimit::windows());

    // assert
    assert_eq!(
        path.to_string_lossy().chars().count(),
        WINDOWS_MAX_PATH_CHARACTERS
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn the_message_of_a_refused_path_names_the_path_the_length_and_the_limit() {
    // arrange
    let path = long_path(WINDOWS_MAX_PATH_CHARACTERS);

    // act
    let error = match check_path_length(&path, PathLengthLimit::windows()) {
        Ok(()) => panic!("the path should be refused"),
        Err(error) => error,
    };
    let message = error.message();

    // assert
    assert!(
        message.contains(&path.to_string_lossy().to_string()),
        "{message}"
    );
    assert!(message.contains(&error.amount().to_string()), "{message}");
    assert!(message.contains("260"), "{message}");
    assert!(message.contains("Windows"), "{message}");
}

#[test]
fn the_message_of_a_refused_path_says_what_the_user_can_do() {
    // arrange
    let path = long_path(WINDOWS_MAX_PATH_CHARACTERS);

    // act
    let error = match check_path_length(&path, PathLengthLimit::windows()) {
        Ok(()) => panic!("the path should be refused"),
        Err(error) => error,
    };

    // assert
    assert!(
        error.advice().contains(&PathAdvice::LongPathsOfTheSystem),
        "{:?}",
        error.advice()
    );
    assert!(
        error.advice().contains(&PathAdvice::ShorterName),
        "{:?}",
        error.advice()
    );
    assert!(
        error.message().contains("LongPathsEnabled"),
        "{}",
        error.message()
    );
}

#[test]
fn the_advice_does_not_ask_for_the_long_paths_when_they_are_already_in_force() {
    // arrange
    let path = long_path(WINDOWS_LONG_PATH_CHARACTERS);

    // act
    let error = match check_path_length(&path, PathLengthLimit::windows_with_long_paths()) {
        Ok(()) => panic!("the path should be refused"),
        Err(error) => error,
    };

    // assert
    assert!(
        !error.advice().contains(&PathAdvice::LongPathsOfTheSystem),
        "{:?}",
        error.advice()
    );
    assert!(error.message().contains("32,767") || error.message().contains("32767"));
}

#[test]
fn the_limit_of_linux_counts_the_bytes_of_the_path() {
    // arrange
    let path = PathBuf::from("/tmp/a-very-long-name-that-does-not-fit-into-the-limit-of-linux");

    // act
    let amount = measure_path_length(&path, LengthUnit::Bytes);

    // assert
    assert_eq!(amount, path.as_os_str().len());
    assert!(check_path_length(&path, PathLengthLimit::linux()).is_ok());
}
