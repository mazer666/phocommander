//! Criterion benchmark: the reaction of the surface to the user.
//!
//! WHAT: the benchmark measures the two reactions that the user notices
//!       first: the switch between two directories that are already loaded
//!       (NFR-02) and the reaction to a key press in the filter field
//!       (NFR-03).
//! WHY:  the specification makes the target values of the performance section
//!       binding gates: "Die folgenden Zielwerte sind verbindliche Gates der
//!       CI, keine Absichtserklärung." A benchmark that only prints a number
//!       does not hold a gate; a benchmark that fails when the number is too
//!       big does.
//!
//!       Both measurements run against the surface of the interface crate and
//!       not against the raw file system, because these two reactions are the
//!       work of the surface. The reading of the disk is measured by the
//!       benchmark of the directory listing, which is the floor that no
//!       implementation can make faster.
//!
//!       The entries of the selection are made in the memory. A directory of
//!       10,000 entries is measured by the benchmark of the listing, and it
//!       would only slow the measurement of the reaction down without saying
//!       anything about the reaction.
//!
//! EXAMPLE:
//! ```text
//! cargo bench -p phocommander-integration-tests --bench interaction
//! ```

use std::hint::black_box;
use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use phocommander_ui::listing::{Entry, EntryKind};
use phocommander_ui::surface::WorkSurface;

/// The number of entries of a panel.
///
/// WHY: the specification names 10,000 entries as the case of the
///       requirements. The number does not change the shape of the work, and
///       it keeps the two measurements comparable with the listing.
const ENTRY_COUNT: usize = 10_000;

/// The threshold of the specification for the switch between the directories.
const SWITCH_THRESHOLD: Duration = Duration::from_millis(16);

/// The threshold of the specification for the reaction to a key press.
const KEY_THRESHOLD: Duration = Duration::from_millis(50);

/// The entries of a panel.
fn entries(count: usize) -> Vec<Entry> {
    (0..count)
        .map(|index| {
            Entry::new(format!("file-{index:06}.txt"), EntryKind::File).with_size(index as u64)
        })
        .collect()
}

/// The surface with two loaded panels.
fn surface() -> WorkSurface {
    let mut surface = WorkSurface::new(
        std::path::PathBuf::from("/photos/left"),
        std::path::PathBuf::from("/photos/right"),
    );
    surface.active_mut().set_entries(entries(ENTRY_COUNT));
    surface
        .panel_mut(phocommander_ui::panel::PanelSide::Right)
        .set_entries(entries(ENTRY_COUNT));
    surface
}

/// The median of the measured times.
fn median(mut times: Vec<Duration>) -> Duration {
    times.sort();
    times[times.len() / 2]
}

/// Measures the switch between two directories that are already loaded.
fn switch_between_loaded_directories(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("switch_between_loaded_directories");
    group.sample_size(50);

    group.bench_function("switch", |bencher| {
        // WHY: the surface is built outside of the measurement. The building
        //       fills two panels with 10,000 entries and costs more time than
        //       the switch itself — a benchmark that measures the preparation
        //       measures the wrong thing.
        bencher.iter_batched(
            surface,
            |mut surface| {
                let start = Instant::now();
                // WHY: the switch itself is the work of the surface. The
                //       reading of the directory is not part of it, because
                //       the directories are already loaded.
                surface.switch_active();
                black_box(surface.active().row_count());
                start.elapsed()
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // WHY: the gate is measured on the median of the runs, because a single
    //       run can be slowed down by the machine and by the garbage
    //       collector. The median is the number that describes the work.
    let mut times = Vec::new();
    for _ in 0..50 {
        let mut surface = surface();
        let start = Instant::now();
        surface.switch_active();
        black_box(surface.active().row_count());
        times.push(start.elapsed());
    }
    let measured = median(times);
    println!("switch between two loaded directories : {measured:?}");
    assert!(
        measured <= SWITCH_THRESHOLD,
        "the switch between two loaded directories took {measured:?}, \
         the threshold is {SWITCH_THRESHOLD:?}"
    );

    group.finish();
}

/// Measures the reaction to a key press in the filter field.
fn reaction_to_a_key_press(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("reaction_to_a_key_press");
    group.sample_size(50);

    group.bench_function("key", |bencher| {
        // WHY: the surface is built outside of the measurement, for the same
        //       reason as in the measurement of the switch.
        bencher.iter_batched(
            surface,
            |mut surface| {
                let start = Instant::now();
                // WHY: the key press is the work of the user. The reaction
                //       is the time until the surface knows the new text of
                //       the field and the index of the filter is rebuilt.
                surface.active_mut().filter_insert("a");
                black_box(surface.active().row_count());
                start.elapsed()
            },
            criterion::BatchSize::SmallInput,
        );
    });

    let mut times = Vec::new();
    for _ in 0..50 {
        let mut surface = surface();
        let start = Instant::now();
        surface.active_mut().filter_insert("a");
        black_box(surface.active().row_count());
        times.push(start.elapsed());
    }
    let measured = median(times);
    println!("reaction to a key press : {measured:?}");
    assert!(
        measured <= KEY_THRESHOLD,
        "the reaction to a key press took {measured:?}, the threshold is {KEY_THRESHOLD:?}"
    );

    group.finish();
}

criterion_group!(benches, switch_between_loaded_directories, reaction_to_a_key_press);
criterion_main!(benches);
