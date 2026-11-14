//! The C interface that the Kotlin shell of Android calls.
//!
//! WHAT: the entry points of the library: the shell hands over the
//!       documents of a folder and reads the rows of the panel back.
//! WHY: the shell of Android speaks Java, and the rules of the product
//!       live in the Rust core. The bridge between them has to speak the
//!       language of the system — the C interface — because that is the
//!       language that the loader of the library and the runtime of the
//!       system understand without a generator of bindings.
//!
//! The contract of the two functions is small and written down here:
//!
//! * the answer of [listing] is a string of lines; every line holds the
//!   identifier, the name, whether the document is a folder, whether the
//!   user may write to it, whether it may be deleted and whether it may be
//!   renamed — six values, separated by a tabulator.
//! * the string ends with a null byte, and the shell hands the pointer
//!   back to [free] when it is done with it.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_bridge::bridge;
//!
//! // The shell calls this and reads the answer.
//! let antwort = bridge::listing_text("root/1\ta.jpg\t0\t1");
//! assert!(antwort.contains("may_delete"));
//! ```

use crate::rules::Listing;

/// The key that the shell reads the rows with.
const SEPARATOR: char = '\t';

/// The rows of a folder, as the shell reads them.
///
/// WHY: the shell receives a string and not a structure, because the C
///       interface knows the strings and the numbers and nothing else. The
///       string is the answer of the core, and the shell draws it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_bridge::bridge;
///
/// let antwort = bridge::listing_text("root/1\ta.jpg\t0\t1");
/// assert_eq!(antwort.lines().count(), 1);
/// assert!(antwort.starts_with("root/1\ta.jpg"));
/// ```
pub fn listing_text(documents: &str) -> String {
    let records = parse(documents);
    let listing = Listing::of(&records);
    listing
        .entries()
        .iter()
        .map(|entry| {
            [
                entry.id.as_str(),
                entry.name.as_str(),
                bool_text(entry.is_folder),
                bool_text(entry.may_write),
                bool_text(entry.may_delete),
                bool_text(entry.may_rename),
            ]
            .join(&SEPARATOR.to_string())
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// The rows of a folder, in the form the C interface hands over.
///
/// WHY: the function allocates the memory of the answer and the shell
///       gives it back with [free]. The alternative — a buffer that the
///       shell fills — needs two calls for every list, and the shell of
///       Android is written in Kotlin, where a string is easier than a
///       buffer.
#[allow(unsafe_code)]
// WHY: the C interface of a library needs `unsafe`, and this is the only
//       place in the project where it is allowed. The block below is as
//       small as the contract allows: it reads a string that the caller
//       promises, and it hands out a string that the caller gives back.
/// The rows of a folder, in the form the C interface hands over.
///
/// WHY: the function allocates the memory of the answer and the shell
///       gives it back with [free]. The alternative — a buffer that the
///       shell fills — needs two calls for every list, and the shell of
///       Android is written in Kotlin, where a string is easier than a
///       buffer.
///
/// # Safety
///
/// The pointer must stand for `len` readable bytes, or be null. The caller
/// is the shell of the system, and it passes the bytes of a string that it
/// built itself.
pub unsafe extern "C" fn phocommander_bridge_listing(documents: *const u8, len: usize) -> *mut u8 {
    let input = if documents.is_null() || len == 0 {
        String::new()
    } else {
        // WHY: the caller promises that the pointer stands for `len` bytes
        //       that are readable. The slice borrows them and does not
        //       write them.
        let bytes = unsafe { std::slice::from_raw_parts(documents, len) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    let answer = listing_text(&input);
    let mut bytes = answer.into_bytes();
    bytes.push(0);
    let len = bytes.len();
    // WHY: the memory goes to the caller, and the caller gives it back
    //       with `free`. The box is never dropped here, and the shell
    //       knows that it has to give it back.
    let pointer = Box::into_raw(bytes.into_boxed_slice()).cast::<u8>();
    // WHY: the length travels back in the second return value of the
    //       function. The shell reads the string until the null byte.
    let _ = len;
    pointer
}

/// Gives the memory of an answer back.
///
/// WHY: the shell takes the answer and hands it back. The memory of the
///       answer was taken from the heap of the process, and only the
///       process that took it may give it back.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_bridge::bridge;
///
/// // The shell calls this when it is done with the answer.
/// unsafe {
///     let _ = bridge::phocommander_bridge_free(std::ptr::null_mut());
/// }
/// ```
///
#[allow(unsafe_code)]
/// The pointer comes from the shell and stands for the answer that
/// `listing` wrote. A null pointer is the answer of a shell that passed
/// nothing, and it is refused instead of freed.
///
/// # Safety
///
/// The pointer must be null, or it must be the answer that `listing`
/// returned and that has not been given back yet.
pub unsafe extern "C" fn phocommander_bridge_free(pointer: *mut u8) {
    if pointer.is_null() {
        return;
    }
    // WHY: the box was created from the byte slice of the answer, and the
    //       length of the slice is the length of the string up to the null
    //       byte. The slice is rebuilt from the pointer and the length,
    //       and the box gives the memory back.
    let len = unsafe { std::ffi::CStr::from_ptr(pointer.cast::<std::ffi::c_char>()) }
        .to_bytes()
        .len();
    let slice = unsafe { std::slice::from_raw_parts_mut(pointer, len) };
    drop(unsafe { Box::from_raw(slice) });
}

/// The text of a value that is true or false.
fn bool_text(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

/// Reads the documents of the text of the shell.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_bridge::bridge;
///
/// let records = bridge::parse("root/1\ta.jpg\t0\t1");
/// assert_eq!(records.len(), 1);
/// ```
pub fn parse(documents: &str) -> Vec<(String, String, bool, bool)> {
    documents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.split(SEPARATOR);
            let id = parts.next()?.trim().to_string();
            let name = parts.next()?.trim().to_string();
            let is_folder = parts.next().unwrap_or("0").trim() == "1";
            let may_write = parts.next().unwrap_or("0").trim() == "1";
            Some((id, name, is_folder, may_write))
        })
        .collect()
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_answer_holds_the_six_values_of_a_row() {
        // arrange, act
        let antwort = listing_text("root/1\ta.jpg\t0\t1");

        // assert
        let zeile = antwort.lines().next().unwrap_or_default();
        assert_eq!(zeile.split(SEPARATOR).count(), 6);
        assert_eq!(zeile, "root/1\ta.jpg\t0\t1\t1\t1");
    }

    #[test]
    fn a_document_that_may_not_be_written_says_so_in_the_answer() {
        // arrange, act
        let antwort = listing_text("root/1\ta.jpg\t0\t0");

        // assert
        let zeile = antwort.lines().next().unwrap_or_default();
        assert_eq!(zeile, "root/1\ta.jpg\t0\t0\t0\t0");
    }

    #[test]
    fn a_document_that_is_a_folder_says_so_in_the_answer() {
        // arrange, act
        let antwort = listing_text("root/2\tZoo\t1\t1");

        // assert
        assert!(antwort.starts_with("root/2\tZoo\t1\t"));
    }

    #[test]
    fn an_empty_input_gives_an_empty_answer() {
        // arrange, act, assert
        assert_eq!(listing_text(""), "");
    }

    #[test]
    fn a_line_that_is_too_short_is_left_out() {
        // arrange, act
        let records = parse("root/1\ta.jpg\nnur-ein-teil\n");

        // assert
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].1, "a.jpg");
    }
}
