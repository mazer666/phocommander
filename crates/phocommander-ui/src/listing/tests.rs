// The tests of the content of a directory.
//
// WHY: the tests live in a file of their own, so that the state itself stays
// readable. They may fail loudly when their arrangement is wrong — the rule
// that forbids a panic applies to the production code, and it is enforced
// there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::listing::EntryKind;

/// Builds a listing with the given number of entries.
fn listing(count: usize) -> ListingState {
    let mut listing = ListingState::new(PathBuf::from("/tmp"));
    listing.set_entries(
        (0..count)
            .map(|index| Entry::new(format!("file-{index:06}.txt"), EntryKind::File))
            .collect(),
    );
    listing
}

#[test]
fn an_empty_listing_has_no_entries() {
    // act
    let listing = ListingState::new(PathBuf::from("/tmp"));

    // assert
    assert!(listing.is_empty());
    assert_eq!(listing.len(), 0);
}

#[test]
fn only_the_visible_slice_is_handed_to_the_widgets() {
    // WHY: FR-106 — a directory with a hundred thousand entries must not
    // cost a hundred thousand rows per frame.
    // arrange
    let listing = listing(100_000);

    // act
    let rows: Vec<_> = listing.visible_rows(0, 40).collect();

    // assert
    assert_eq!(rows.len(), 40);
    assert_eq!(listing.rows_materialised(), 40);
    assert_eq!(listing.len(), 100_000);
}

#[test]
fn the_visible_slice_starts_where_it_is_asked_to_start() {
    // arrange
    let listing = listing(1_000);

    // act
    let rows: Vec<_> = listing.visible_rows(100, 10).collect();

    // assert
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].offset(), 100);
    assert_eq!(listing.rows_materialised(), 10);
}

#[test]
fn a_visible_slice_beyond_the_end_is_shortened() {
    // arrange
    let listing = listing(5);

    // act
    let rows: Vec<_> = listing.visible_rows(3, 10).collect();

    // assert
    assert_eq!(rows.len(), 2);
}

#[test]
fn the_order_of_the_screen_is_a_permutation_of_the_entries() {
    // arrange
    let listing = listing(50);

    // act
    let mut positions: Vec<usize> = (0..50)
        .map(|row| listing.position_of_view_row(row).unwrap_or(usize::MAX))
        .collect();

    // assert
    positions.sort_unstable();
    assert_eq!(positions, (0..50).collect::<Vec<usize>>());
}

#[test]
fn the_position_of_an_entry_survives_a_change_of_the_order() {
    // WHY: the selection holds positions, so that it keeps its meaning
    // when the order of the screen changes.
    // arrange
    let mut listing = listing(100);
    let position = listing.position_of_view_row(10).unwrap_or(0);

    // act
    listing.set_sort(Sort::by(SortKey::Size));

    // assert
    let view_row = listing.view_row_of_position(position);
    assert!(view_row.is_some());
    assert_eq!(
        listing.position_of_view_row(view_row.unwrap_or(0)),
        Some(position)
    );
}

#[test]
fn the_order_by_name_is_the_order_of_the_names() {
    // arrange
    let mut listing = ListingState::new(PathBuf::from("/tmp"));
    listing.set_entries(vec![
        Entry::new("b.txt", EntryKind::File),
        Entry::new("a.txt", EntryKind::File),
        Entry::new("c.txt", EntryKind::File),
    ]);

    // act
    let names: Vec<String> = listing
        .visible_rows(0, 3)
        .map(|row| row.name().to_string())
        .collect();

    // assert
    assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);
}

#[test]
fn writes_a_size_the_way_a_file_manager_writes_it() {
    // act
    let sizes = [
        format_size(0),
        format_size(512),
        format_size(2_048),
        format_size(5_368_709_120),
    ];

    // assert
    assert_eq!(sizes[0], "0 B");
    assert_eq!(sizes[1], "512 B");
    assert_eq!(sizes[2], "2.0 KiB");
    assert_eq!(sizes[3], "5.0 GiB");
}

#[test]
fn writes_a_date_the_way_a_file_manager_writes_it() {
    // arrange
    let moment = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_772_000_000);

    // act
    let text = format_time(moment);

    // assert
    assert_eq!(text.len(), 16);
    assert!(text.starts_with("2026-"));
}
