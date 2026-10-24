//! Criterion benchmark: the directory listing of a file panel.
//!
//! WHAT: the benchmark builds a directory with 10,000 entries in a temporary
//!       directory and measures the work a file panel has to do to show it:
//!       reading the directory, collecting the entries, reading the metadata
//!       of every entry and sorting the result by name. It also checks the
//!       threshold of the specification: the listing of 10,000 entries has to
//!       be ready in less than 100 milliseconds.
//! WHY:  the specification makes the target values of the performance section
//!       binding gates: "Die folgenden Zielwerte sind verbindliche Gates der
//!       CI, keine Absichtserklärung. Ein Meilenstein gilt erst als erreicht,
//!       wenn die Messung vorliegt." A benchmark that only prints a number
//!       does not hold a gate; a benchmark that fails when the number is too
//!       big does.
//!
//!       The measurement is taken on the raw file system access, because that
//!       is the part no implementation can make faster: the entries have to be
//!       read from the disk. It is therefore the floor that the listing of
//!       `phocommander-core` has to reach, and the reference against which
//!       the product measurement is compared once the directory model of the
//!       core crate exists.
//!
//!       The directory is built in the temporary directory of the system and
//!       removed afterwards. A benchmark that leaves 10,000 files behind is a
//!       benchmark that fills the disk of the machine that runs it.
//!
//! EXAMPLE:
//! ```text
//! cargo bench -p phocommander-integration-tests --bench directory_listing
//! ```

use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};

/// The number of entries the directory of the benchmark holds.
///
/// WHY: the specification names 10,000 entries as the case of the
/// requirement "Verzeichnisliste bis 10.000 Einträge < 100 ms".
const ENTRY_COUNT: usize = 10_000;

/// The threshold of the specification for this measurement.
const THRESHOLD: Duration = Duration::from_millis(100);

/// One entry of the benchmark directory.
struct Entry {
    name: String,
    size: u64,
    is_directory: bool,
}

/// Builds the directory of the benchmark and removes it afterwards.
struct BenchmarkDirectory {
    path: PathBuf,
}

impl BenchmarkDirectory {
    /// Creates a directory with `ENTRY_COUNT` files and subdirectories.
    fn create() -> Self {
        let path = std::env::temp_dir().join(format!(
            "phocommander-bench-{}-{}",
            std::process::id(),
            ENTRY_COUNT
        ));
        let _ = fs::remove_dir_all(&path);
        if let Err(error) = fs::create_dir_all(&path) {
            panic!("the benchmark directory {path:?} could not be created: {error}");
        }
        for index in 0..ENTRY_COUNT {
            let name = path.join(format!("entry-{index:05}.dat"));
            if let Err(error) = fs::write(&name, b"phocommander benchmark payload") {
                panic!("the benchmark file {name:?} could not be written: {error}");
            }
        }
        // Every tenth entry is a directory, so that the benchmark does not
        // measure a flat list only.
        for index in 0..(ENTRY_COUNT / 10) {
            let name = path.join(format!("directory-{index:05}"));
            if let Err(error) = fs::create_dir_all(&name) {
                panic!("the benchmark directory {name:?} could not be created: {error}");
            }
        }
        Self { path }
    }

    /// Returns the path of the directory.
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for BenchmarkDirectory {
    /// Removes the directory of the benchmark.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Reads the directory and returns the entries sorted by name.
fn list_entries(directory: &Path) -> Vec<Entry> {
    let read = match fs::read_dir(directory) {
        Ok(read) => read,
        Err(error) => panic!("the directory {directory:?} could not be read: {error}"),
    };
    let mut entries: Vec<Entry> = Vec::with_capacity(ENTRY_COUNT);
    for entry in read {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => panic!("an entry of {directory:?} could not be read: {error}"),
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => panic!("the metadata of {name:?} could not be read: {error}"),
        };
        entries.push(Entry {
            name,
            size: metadata.len(),
            is_directory: metadata.is_dir(),
        });
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries
}

/// Measures the listing a number of times and returns the median duration.
fn measure_median(directory: &Path, runs: usize) -> Duration {
    let mut durations: Vec<Duration> = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        let entries = list_entries(directory);
        durations.push(start.elapsed());
        black_box(summary(&entries));
    }
    durations.sort();
    durations[durations.len() / 2]
}

/// Returns the values of the listing that the panel would draw.
///
/// WHY: the benchmark has to keep the work it measures. Without the summary
/// the compiler is free to drop the size and the kind of every entry, and the
/// measurement would show a listing that no panel ever draws.
fn summary(entries: &[Entry]) -> (u64, usize, usize) {
    let total_size: u64 = entries.iter().map(|entry| entry.size).sum();
    let directories = entries.iter().filter(|entry| entry.is_directory).count();
    (total_size, directories, entries.len())
}

/// The benchmark of the directory listing.
fn directory_listing(criterion: &mut Criterion) {
    let directory = BenchmarkDirectory::create();

    let mut group = criterion.benchmark_group("directory_listing");
    group.sample_size(20);
    group.bench_function("list_10_000_entries", |bencher| {
        bencher.iter(|| list_entries(directory.path()));
    });
    group.finish();

    // The threshold check. The measurement of the criterion run above is the
    // detailed result; this is the gate: the listing of 10,000 entries has to
    // be ready in less than 100 milliseconds, otherwise the milestone is not
    // reached.
    let median = measure_median(directory.path(), 5);
    println!(
        "directory listing of {ENTRY_COUNT} entries: median {median:?} (threshold {THRESHOLD:?})"
    );
    assert!(
        median < THRESHOLD,
        "the listing of {ENTRY_COUNT} entries took {median:?}, the threshold is {THRESHOLD:?}"
    );
}

criterion_group!(benches, directory_listing);
criterion_main!(benches);
