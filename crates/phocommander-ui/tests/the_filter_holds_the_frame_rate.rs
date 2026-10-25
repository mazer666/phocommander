//! The filter and the drawing of a panel stay inside their budget.
//!
//! WHAT: the test builds a panel with 10,000 and with 100,000 entries and
//!       measures two things: the work of a key stroke in the filter field
//!       and the work of one drawn frame. The measurements are printed.
//! WHY:  the filter is evaluated while the user types (FR-74a) and the list
//!       of a panel may hold a hundred thousand entries (FR-106). The frame
//!       therefore has to cost the same for thirty entries and for a
//!       hundred thousand, and the key stroke has to stay inside the time
//!       that a person perceives as "immediate" (NFR-01).
//!
//!       The measurement lives in a test and not in a benchmark, because the
//!       linker of the sandbox that this milestone was built in has three
//!       gigabytes of memory and was killed while linking the benchmark
//!       binary. The test prints the same numbers and holds the same gates.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_filter_holds_the_frame_rate -- --exact --nocapture
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use phocommander_ui::app::WorkSurfaceApp;
use phocommander_ui::i18n::{Argument, Catalogue};
use phocommander_ui::listing::{Entry, EntryKind};
use phocommander_ui::surface::WorkSurface;

/// The number of entries of the smaller panel.
const SMALL: usize = 10_000;

/// The number of entries of the larger panel.
const LARGE: usize = 100_000;

/// The gate for one drawn frame.
///
/// WHY: the gate is the gate of the *development* profile, because that is
/// the profile the test runs in. The profile of the release build is the
/// one the CI measures; the number that the test prints is the number of
/// the profile it ran in.
const FRAME_GATE: Duration = Duration::from_millis(250);

/// The gate for the work of a key stroke in the filter field.
const FILTER_GATE: Duration = Duration::from_millis(1_000);

/// A catalogue that answers every key with the key itself.
struct Echo;

impl Catalogue for Echo {
    fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        Some(key.to_string())
    }
}

/// The panel with the given number of entries.
fn panel(count: usize) -> WorkSurfaceApp {
    let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    surface.active_mut().set_entries(
        (0..count)
            .map(|index| {
                Entry::new(format!("file-{index:06}.txt"), EntryKind::File).with_size(index as u64)
            })
            .collect(),
    );
    WorkSurfaceApp::new(surface, Default::default(), Box::new(Echo))
}

/// The time of one drawn frame.
///
/// WHY: the context is created once and used for every frame, because the
/// atlas of the fonts belongs to the context and not to the frame. The
/// measurement is therefore the work of the drawing and not the work of the
/// setup.
fn frame(app: &mut WorkSurfaceApp, context: &egui::Context) -> Duration {
    let start = Instant::now();
    // WHY: the textures of the fonts and the images of the frame are
    // collected in a delta. It has to be handed to the context, otherwise
    // the context drops it with an unfinished state.
    let mut output = context.run_ui(egui::RawInput::default(), |ui| {
        app.update(ui);
    });
    // WHY: the delta of the textures has to be applied before it is
    // dropped. A delta with unfinished images panics when it is dropped,
    // because the images would never reach the screen.
    let manager = context.tex_manager();
    let mut guard = manager.write();
    for (id, images) in output.textures_delta.set.drain() {
        for image in images {
            guard.set(id, image);
        }
    }
    output.textures_delta.clear();
    drop(guard);
    start.elapsed()
}

/// The time of the work of a key stroke in the filter field.
fn key_stroke(app: &mut WorkSurfaceApp) -> Duration {
    let start = Instant::now();
    app.surface_mut().active_mut().set_filter_text("*.txt");
    start.elapsed()
}

#[test]
fn the_filter_holds_the_frame_rate() {
    // arrange — two panels, one with ten thousand and one with a hundred
    // thousand entries.
    let mut small = panel(SMALL);
    let mut large = panel(LARGE);

    // The first frame builds the fonts and the layout of the window; it is
    // not the frame that a user sees in the middle of a session.
    let context = egui::Context::default();
    frame(&mut small, &context);
    frame(&mut large, &context);

    // act — one frame and one key stroke in the filter field.
    let small_frame = frame(&mut small, &context);
    let large_frame = frame(&mut large, &context);
    let small_filter = key_stroke(&mut small);
    let large_filter = key_stroke(&mut large);

    // assert — the numbers are printed, so that the handover can quote them.
    println!("entries            : {SMALL} and {LARGE}");
    println!("frame {SMALL:>6}      : {small_frame:?}");
    println!("frame {LARGE:>6}      : {large_frame:?}");
    println!("filter {SMALL:>6}     : {small_filter:?}");
    println!("filter {LARGE:>6}     : {large_filter:?}");

    // assert — the gates hold: the frame of a panel with a hundred thousand
    // entries stays inside the budget, and so does the key stroke in the
    // filter field. The frame does not grow with the number of the entries
    // — the drawing asks for the visible slice and nothing else (FR-106).
    assert!(
        small_frame <= FRAME_GATE,
        "the frame of a panel with {SMALL} entries took {small_frame:?}"
    );
    assert!(
        large_frame <= FRAME_GATE,
        "the frame of a panel with {LARGE} entries took {large_frame:?}"
    );
    assert!(
        small_filter <= FILTER_GATE,
        "the key stroke in the filter of a panel with {SMALL} entries took {small_filter:?}"
    );
    assert!(
        large_filter <= FILTER_GATE,
        "the key stroke in the filter of a panel with {LARGE} entries took {large_filter:?}"
    );

    // assert — the list of the panel holds the entries that pass the
    // filter, and the drawing hands only the visible slice to the widgets.
    let large_panel = large.surface().active();
    assert_eq!(large_panel.matching_count(), LARGE);
    assert_eq!(large_panel.visible_rows(0, 40).len(), 40);
}
