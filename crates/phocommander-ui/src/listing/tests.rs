// The tests of the content of a directory.
//
// WHY: the tests live in a file of their own, so that the state itself stays
// readable. They may fail loudly when their arrangement is wrong — the rule
// that forbids a panic applies to the production code, and it is enforced
// there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::format::PlainFormats;
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
fn writes_a_size_the_way_the_plain_formats_write_it() {
    // act
    let sizes = [
        PlainFormats.size(0),
        PlainFormats.size(512),
        PlainFormats.size(2_048),
        PlainFormats.size(5_368_709_120),
    ];

    // assert
    assert_eq!(sizes[0], "0 B");
    assert_eq!(sizes[1], "512 B");
    assert_eq!(sizes[2], "2.0 KiB");
    assert_eq!(sizes[3], "5.0 GiB");
}

#[test]
fn the_size_of_a_row_follows_the_formats_of_the_interface() {
    // WHY: FR-123 — the size of a row is written the way the locale of the
    //       user writes it. The row therefore asks the formats and knows no
    //       rule of a locale itself.
    // arrange
    let mut listing = ListingState::new(PathBuf::from("/tmp"));
    listing.set_entries(vec![
        Entry::new("notes.txt", EntryKind::File).with_size(1_200_000),
    ]);

    // act
    let rows: Vec<Row<'_>> = listing.visible_rows(0, 1).collect();

    // assert
    assert_eq!(rows[0].size_text(&PlainFormats), "1.1 MiB");
    assert_eq!(rows[0].size_text(&German), "1,2 MB");
}

#[test]
fn writes_a_date_the_way_the_plain_formats_write_it() {
    // arrange
    let moment = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_772_000_000);

    // act
    let text = PlainFormats.date_time(moment);

    // assert
    assert_eq!(text.len(), 16);
    assert!(text.starts_with("2026-"));
}

/// The formats of a German interface.
///
/// WHY: the formats are the seam of the interface. A test that wants a
///       German size writes one itself and does not need the crate of the
///       localisation — the crate rule forbids the dependency.
struct German;

impl Formats for German {
    fn number(&self, value: u64) -> String {
        value.to_string()
    }

    fn size(&self, bytes: u64) -> String {
        let megabytes = bytes as f64 / 1_000_000.0;
        // WHY: the German locale writes the separator of the decimals with a
        //       comma. The rule belongs to the implementation of the locale
        //       and not to the interface, so the test writes it here.
        let number = format!("{megabytes:.1}").replace('.', ",");
        format!("{number} MB")
    }

    fn date(&self, _moment: SystemTime) -> String {
        String::new()
    }

    fn time(&self, _moment: SystemTime) -> String {
        String::new()
    }
}
