//! The Windows port of Phocommander.
//!
//! WHAT: the facts of the Windows build — the manifest that declares the
//!       long path support ([{manifest_path}], [{manifest_text}],
//!       [{declares_long_paths}]), the limit of the length of a path
//!       ([{path_length_limit}]) and the resource that carries the manifest
//!       into the program ([{resource}]).
//! WHY: Windows limits a path to 260 characters, and only a program that
//!       declares `longPathAware` in its manifest may use the longer paths
//!       of the system. The declaration is a resource of the executable, and
//!       the build script writes it there. The facts live here, so that the
//!       port can be read in one place and can be proven without a Windows
//!       machine.
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::windows::path_length_limit;
//!
//! // WHY: without the setting of the system, Windows accepts 260 characters.
//! let limit = path_length_limit(false);
//! assert_eq!(limit.amount(), 260);
//! ```

use std::path::{Path, PathBuf};

use phocommander_core::path::PathLengthLimit;

/// The name of the file of the manifest of the application.
pub const MANIFEST_FILE_NAME: &str = "windows.manifest";

/// The name of the element of the manifest that declares the long paths.
pub const LONG_PATH_AWARE_ELEMENT: &str = "longPathAware";

/// The beginning of a comment of a document.
const COMMENT_START: &str = "<!--";

/// The end of a comment of a document.
const COMMENT_END: &str = "-->";

/// The resource that carries the manifest into the program.
#[path = "../build/resource.rs"]
pub mod resource;

/// The path of the manifest of the application.
///
/// WHY: the manifest lies next to the `Cargo.toml` of the application, and
///       the build script reads it from there. The function returns the path
///       so that a test can read the very file that the build uses.
pub fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(MANIFEST_FILE_NAME)
}

/// The text of the manifest of the application, if the file can be read.
///
/// WHY: the program does not read the manifest at run time — the manifest is
///       a resource of the program, and the loader of Windows reads it. The
///       function serves the tests and the tools that check the build.
pub fn manifest_text() -> Option<String> {
    std::fs::read_to_string(manifest_path()).ok()
}

/// Whether a manifest declares the long path support.
///
/// WHY: the declaration is the element `longPathAware` with the value
///       `true`. The function reads the text and does not build a tree of
///       the document, because the file is ours and the only question is
///       whether that one element is there.
pub fn declares_long_paths(manifest: &str) -> bool {
    let manifest = without_comments(manifest);

    let Some(start) = manifest.find(LONG_PATH_AWARE_ELEMENT) else {
        return false;
    };

    let after_the_name = &manifest[start + LONG_PATH_AWARE_ELEMENT.len()..];
    let Some(end_of_the_name) = after_the_name.find('>') else {
        return false;
    };

    let content = &after_the_name[end_of_the_name + 1..];
    let Some(end_of_the_content) = content.find('<') else {
        return false;
    };

    content[..end_of_the_content]
        .trim()
        .eq_ignore_ascii_case("true")
}

/// The document without its comments.
///
/// WHY: a comment of the manifest may name the element, and a reader that
///       looks for the element must not find the name of the comment.
fn without_comments(document: &str) -> String {
    let mut result = String::with_capacity(document.len());
    let mut rest = document;

    while let Some(start) = rest.find("<!--") {
        result.push_str(&rest[..start]);

        let Some(end) = rest[start + COMMENT_START.len()..].find(COMMENT_END) else {
            return result;
        };

        rest = &rest[start + COMMENT_START.len() + end + COMMENT_END.len()..];
    }

    result.push_str(rest);
    result
}

/// The limit of the length of a path on Windows.
///
/// WHY: Windows has two limits — 260 characters, and 32,767 characters when
///       the setting `LongPathsEnabled` of the system is 1 and the program
///       declares that it can handle long paths. The program asks the system
///       for the setting and gives the answer here, so that the rule of the
///       limit stays in the core and the reading of the setting stays in the
///       port.
pub fn path_length_limit(long_paths_of_the_system: bool) -> PathLengthLimit {
    if long_paths_of_the_system {
        PathLengthLimit::windows_with_long_paths()
    } else {
        PathLengthLimit::windows()
    }
}

#[cfg(test)]
mod tests {
    // WHY: the tests of the module live in the module itself, because the
    //       module is small and its proof belongs to it.
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    /// Reads a number of two bytes out of a buffer.
    fn read_u16(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
    }

    /// Reads a number of four bytes out of a buffer.
    fn read_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ])
    }

    #[test]
    fn the_manifest_of_the_repository_declares_the_long_paths() {
        // arrange
        let path = manifest_path();

        // act
        let text = match manifest_text() {
            Some(text) => text,
            None => panic!("the manifest {} could not be read", path.display()),
        };

        // assert
        assert!(declares_long_paths(&text), "{}", text);
    }

    #[test]
    fn the_manifest_of_the_repository_asks_for_no_elevation() {
        // arrange
        let text = match manifest_text() {
            Some(text) => text,
            None => panic!("the manifest could not be read"),
        };

        // act and assert
        // WHY: the program never asks for elevated rights (NFR-23), and the
        //       manifest says so to the loader of Windows.
        assert!(text.contains("requestedExecutionLevel"), "{}", text);
        assert!(text.contains("level=\"asInvoker\""), "{}", text);
    }

    #[test]
    fn a_manifest_without_the_element_does_not_declare_the_long_paths() {
        // act and assert
        assert!(!declares_long_paths("<assembly/>"));
        assert!(!declares_long_paths(
            "<assembly><application><windowsSettings><longPathAware>false</longPathAware>\
             </windowsSettings></application></assembly>"
        ));
        assert!(!declares_long_paths("<assembly><longPathAware>"));
    }

    #[test]
    fn a_manifest_with_the_element_and_the_value_true_declares_the_long_paths() {
        // act and assert
        assert!(declares_long_paths(
            "<asmv3:application><asmv3:windowsSettings \
             xmlns:ws2=\"http://schemas.microsoft.com/SMI/2016/WindowsSettings\">\
             <ws2:longPathAware>true</ws2:longPathAware></asmv3:windowsSettings></asmv3:application>"
        ));
    }

    #[test]
    fn the_limit_of_windows_is_the_limit_of_windows() {
        // act
        let without_long_paths = path_length_limit(false);
        let with_long_paths = path_length_limit(true);

        // assert
        assert_eq!(without_long_paths.amount(), 260);
        assert_eq!(with_long_paths.amount(), 32_767);
        assert!(without_long_paths.is_windows_without_long_paths());
        assert!(!with_long_paths.is_windows_without_long_paths());
    }

    #[test]
    fn the_resource_carries_the_manifest_as_the_resource_one_of_the_type_manifest() {
        // arrange
        let manifest = b"<assembly/>";
        let resource = resource::manifest_resource(manifest, 0x8664);

        // act and assert
        // WHY: the tree of the resource begins where the section begins, and
        //       the section begins after the two headers of the file.
        const SECTION: usize = 60;

        assert_eq!(read_u16(&resource, 0), 0x8664);
        assert_eq!(read_u16(&resource, 2), 1);
        assert_eq!(&resource[20..28], b".rsrc\0\0\0");
        assert_eq!(read_u32(&resource, SECTION + 0x10), 24);
        assert_eq!(read_u32(&resource, SECTION + 0x28), 1);
        assert_eq!(read_u32(&resource, SECTION + 0x48), 0x58);
        assert_eq!(read_u32(&resource, SECTION + 0x4c), manifest.len() as u32);
        assert_eq!(&resource[SECTION + 0x58..], manifest);
    }

    #[test]
    fn the_resource_of_a_machine_of_the_target_carries_the_machine() {
        // act
        let resource = resource::manifest_resource(b"<assembly/>", 0x014C);

        // assert
        assert_eq!(read_u16(&resource, 0), 0x014C);
    }

    #[test]
    fn the_machine_numbers_of_the_known_architectures_are_known() {
        // act and assert
        assert_eq!(resource::machine_of("x86_64"), Some(0x8664));
        assert_eq!(resource::machine_of("aarch64"), Some(0xAA64));
        assert_eq!(resource::machine_of("x86"), Some(0x014C));
        assert_eq!(resource::machine_of("arm"), Some(0x01C0));
        assert_eq!(resource::machine_of("sparc"), None);
    }
}
