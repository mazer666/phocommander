// The tests of the seam to the engine that forms the new names.
//
// WHY: the tests live in a file of their own, so that the seam itself stays
// readable. They may fail loudly when their arrangement is wrong — the rule
// that forbids a panic applies to the production code, and it is enforced
// there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

#[test]
fn a_preview_without_a_name_may_not_rename() {
    // arrange
    let mut preview = RenamePreview::empty();
    preview.rows.push(PreviewRow::failed(
        "/photos/a.jpg",
        PreviewProblem::empty_format(),
    ));

    // act
    let may = preview.may_rename();

    // assert
    assert!(!may);
}

#[test]
fn a_preview_with_a_conflict_may_not_rename() {
    // arrange
    let mut preview = RenamePreview::empty();
    preview
        .rows
        .push(PreviewRow::named("/photos/a.jpg", "a_001.jpg"));
    preview.conflicts.push(Conflict::new(
        ConflictKind::TargetExists,
        "/photos/a_001.jpg",
    ));

    // act
    let may = preview.may_rename();

    // assert
    assert!(!may);
}

#[test]
fn a_preview_with_names_and_without_conflicts_may_rename() {
    // arrange
    let mut preview = RenamePreview::empty();
    preview
        .rows
        .push(PreviewRow::named("/photos/a.jpg", "a_001.jpg"));

    // act
    let may = preview.may_rename();

    // assert
    assert!(may);
}

#[test]
fn every_kind_of_a_problem_has_a_key_of_its_own() {
    // act
    let keys = [
        ProblemKind::EmptyFormat.message_key(),
        ProblemKind::UnknownValue.message_key(),
        ProblemKind::InvalidPattern.message_key(),
        ProblemKind::InvalidName.message_key(),
    ];

    // assert
    assert_eq!(keys.len(), 4);
    assert_ne!(keys[0], keys[1]);
    assert_ne!(keys[2], keys[3]);
}

#[test]
fn every_kind_of_a_conflict_has_a_key_of_its_own() {
    // act
    let keys = [
        ConflictKind::TargetExists.message_key(),
        ConflictKind::DuplicateTarget.message_key(),
        ConflictKind::MissingSource.message_key(),
    ];

    // assert
    assert_eq!(keys.len(), 3);
    assert_ne!(keys[0], keys[1]);
    assert_ne!(keys[1], keys[2]);
}
