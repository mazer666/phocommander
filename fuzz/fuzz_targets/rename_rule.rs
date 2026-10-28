#![no_main]

//! The fuzzing of the rules of the rename.
//!
//! WHAT: the target draws a rule and a name of a file, and hands both to
//!       the engine of the rename.
//! WHY: the user writes the search text and the replacement, and the
//!       engine has to survive whatever they write. A rule that panics
//!       would take the whole rename with it, and a rule that loses the
//!       text would produce names that the user never asked for.
//!
//!       The property test of the same rule proves the behaviour on the
//!       stable compiler and in every build. The fuzzer looks for the
//!       input that the property test cannot draw — the one that makes the
//!       regular expression engine work, or the one that makes it give up.
//!
//! EXAMPLE:
//! ```text
//! cargo +nightly fuzz run rename_rule
//! ```

use libfuzzer_sys::fuzz_target;
use phocommander_ops::template::{RenameInput, RenameRule, ReplaceRule, form_names};
use std::time::{Duration, UNIX_EPOCH};

/// The rule that the fuzzer builds out of the bytes.
///
/// WHY: the bytes of the fuzzer are not necessarily valid UTF-8, and the
///       engine takes a text. The bytes that are not valid UTF-8 become the
///       replacement character, so that the target still reaches the engine
///       with the length and the structure of the input.
fn to_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fuzz_target!(|data: &[u8]| {
    // WHY: the input is cut into three parts — the search text, the
    //       replacement and the name of the file. A cut that is too small
    //       leaves nothing to draw, and the target would measure the
    //       emptiness instead of the engine.
    if data.len() < 3 {
        return;
    }
    let grenze = data.len() / 3;
    let suche = to_text(&data[..grenze]);
    let ersetzung = to_text(&data[grenze..2 * grenze]);
    let name = to_text(&data[2 * grenze..]);

    // WHY: the name of the file is a part of the path, because that is
    //       where the engine reads it from.
    let pfad = std::path::PathBuf::from(format!("/photos/{name}"));

    // WHY: the rule is built twice — once as a literal search and once as
    //       a regular expression. The second one is the interesting one,
    //       because the pattern comes from the user.
    for is_regex in [false, true] {
        let regel = RenameRule::new("[name]_[n].[ext]").with_replacement(ReplaceRule {
            search: suche.clone(),
            replacement: ersetzung.clone(),
            regex: is_regex,
        });

        let input = RenameInput::new(pfad.clone())
            .with_size(1_024)
            .with_checksum(42);

        // WHY: the engine may refuse the rule — a pattern that it cannot
        //       read is an error and not a panic. Both are fine; what must
        //       not happen is that the program stops.
        let _ = form_names(
            &regel,
            &[input],
            UNIX_EPOCH + Duration::from_secs(1_700_000_000),
        );
    }
});
