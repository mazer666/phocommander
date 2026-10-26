//! Tell cargo that the catalogues are part of the build.
//!

// WHAT: this build script prints the instruction that the files below
//       `locales/` are watched by cargo.
// WHY: the crate `rust-embed` embeds the catalogues of the languages into the
//       binary when the crate is compiled, and it does not tell cargo that
//       those files matter. Cargo watches the sources of the crate, and a
//       catalogue is not a source. A translation that is corrected would
//       therefore stay invisible in the binary until someone cleans the crate
//       by hand — and the interface would keep showing the old text.
//
//       The instruction below makes cargo rebuild the crate when a catalogue
//       changes, and the change is then embedded again.
//
// EXAMPLE:
//     cargo build -p phocommander-i18n
//
// The script changes nothing in the repository. It only tells cargo what to
// watch.

fn main() {
    // WHY: the whole directory, and not the files of one language. A new
    //       language is a new directory, and it has to be embedded as well.
    println!("cargo:rerun-if-changed=locales");
}
