//! The resource file that carries the application manifest into the program.
//!
//! WHAT: a function that writes the manifest of the application into a
//!       resource file of the format that `cvtres.exe` writes
//!       ([{manifest_resource}]) and the number of the machine that the
//!       header of such a file carries ([{machine_of}]).
//! WHY: the manifest of an application is a *resource* of the executable,
//!       and the linker takes such a resource as a file of its own. The
//!       file is written here and not with a tool of the Windows SDK, so
//!       that the build needs nothing but the compiler and the linker, and
//!       so that the result is the same on every machine.
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::windows::resource::manifest_resource;
//!
//! let resource = manifest_resource(b"<assembly/>", 0x8664);
//! assert!(!resource.is_empty());
//! ```

/// The type of the resource that carries a manifest (RT_MANIFEST).
///
/// WHY: the loader of Windows looks for the manifest of a program in the
///       resources of the type 24.
pub const MANIFEST_RESOURCE_TYPE: u32 = 24;

/// The number of the resource that carries the manifest.
///
/// WHY: the number 1 is the one that the loader looks for
///       (CREATEPROCESS_MANIFEST_RESOURCE_ID).
pub const MANIFEST_RESOURCE_ID: u32 = 1;

/// The language of the resource (English, United States).
///
/// WHY: the loader reads the resource of the language of the system and
///       falls back to the neutral one. The program has one manifest and
///       one language.
const RESOURCE_LANGUAGE: u32 = 0x0409;

/// The offset of the root directory of the resource tree.
const ROOT_DIRECTORY: usize = 0x00;

/// The offset of the entry of the root directory.
const ROOT_ENTRY: usize = 0x10;

/// The offset of the directory of the type.
const TYPE_DIRECTORY: usize = 0x18;

/// The offset of the entry of the type.
const TYPE_ENTRY: usize = 0x28;

/// The offset of the directory of the number.
const NAME_DIRECTORY: usize = 0x30;

/// The offset of the entry of the number.
const NAME_ENTRY: usize = 0x40;

/// The offset of the entry of the data.
const DATA_ENTRY: usize = 0x48;

/// The offset of the data itself.
const DATA: usize = 0x58;

/// The size of the header of a file of COFF objects.
const FILE_HEADER_SIZE: usize = 20;

/// The size of the header of a section.
const SECTION_HEADER_SIZE: usize = 40;

/// The bit that says that an entry of a directory points to a directory and
/// not to data.
const HIGH_BIT: u32 = 0x8000_0000;

/// Writes the manifest into a resource file.
///
/// WHY: the tree of the resource has three levels — the type 24 holds the
///       number 1, the number 1 holds the language, and the language holds
///       the manifest. The file is a file of COFF objects with one section
///       named `.rsrc`, and the time stamp of its header is zero, so that
///       two builds of the same source produce the same file.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_desktop::windows::resource::manifest_resource;
///
/// let resource = manifest_resource(b"<assembly/>", 0x8664);
/// assert_eq!(&resource[0..2], &0x8664u16.to_le_bytes());
/// ```
pub fn manifest_resource(manifest: &[u8], machine: u16) -> Vec<u8> {
    let mut tree = vec![0u8; DATA];

    // The root directory holds one entry: the type of the manifest.
    put_u32(&mut tree, ROOT_DIRECTORY + 12, 1);
    put_u32(&mut tree, ROOT_ENTRY, MANIFEST_RESOURCE_TYPE);
    put_u32(&mut tree, ROOT_ENTRY + 4, TYPE_DIRECTORY as u32);

    // The directory of the type holds one entry: the number of the manifest.
    put_u32(&mut tree, TYPE_DIRECTORY + 12, 1);
    put_u32(&mut tree, TYPE_ENTRY, MANIFEST_RESOURCE_ID);
    put_u32(&mut tree, TYPE_ENTRY + 4, NAME_DIRECTORY as u32 | HIGH_BIT);

    // The directory of the number holds one entry: the language.
    put_u32(&mut tree, NAME_DIRECTORY + 12, 1);
    put_u32(&mut tree, NAME_ENTRY, RESOURCE_LANGUAGE);
    put_u32(&mut tree, NAME_ENTRY + 4, DATA_ENTRY as u32);

    // The entry of the data says where the data lies and how long it is.
    put_u32(&mut tree, DATA_ENTRY, DATA as u32);
    put_u32(&mut tree, DATA_ENTRY + 4, manifest.len() as u32);

    let mut section = tree;
    section.extend_from_slice(manifest);

    let mut file = Vec::with_capacity(FILE_HEADER_SIZE + SECTION_HEADER_SIZE + section.len());
    file.extend_from_slice(&file_header(machine));
    file.extend_from_slice(&section_header(section.len() as u32));
    file.extend_from_slice(&section);
    file
}

/// The number of the machine, as the header of a file of COFF objects wants it.
///
/// WHY: the linker refuses a file of resources whose machine does not match
///       the machine of the program ("machine type x86 conflicts with x64").
///       The numbers are the ones of the specification of the format.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_desktop::windows::resource::machine_of;
///
/// assert_eq!(machine_of("x86_64"), Some(0x8664));
/// assert_eq!(machine_of("sparc"), None);
/// ```
pub fn machine_of(architecture: &str) -> Option<u16> {
    match architecture {
        "x86_64" => Some(0x8664),
        "aarch64" => Some(0xAA64),
        "x86" => Some(0x014C),
        "arm" => Some(0x01C0),
        _ => None,
    }
}

/// The header of the file: the machine and the one section.
fn file_header(machine: u16) -> [u8; FILE_HEADER_SIZE] {
    let mut header = [0u8; FILE_HEADER_SIZE];
    header[0..2].copy_from_slice(&machine.to_le_bytes());
    header[2..4].copy_from_slice(&1u16.to_le_bytes());
    header
}

/// The header of the section `.rsrc`.
fn section_header(size: u32) -> [u8; SECTION_HEADER_SIZE] {
    let mut header = [0u8; SECTION_HEADER_SIZE];
    header[0..8].copy_from_slice(b".rsrc\0\0\0");
    header[8..12].copy_from_slice(&size.to_le_bytes());
    header[16..20].copy_from_slice(&size.to_le_bytes());
    header[20..24]
        .copy_from_slice(&((FILE_HEADER_SIZE + SECTION_HEADER_SIZE) as u32).to_le_bytes());
    header[36..40].copy_from_slice(&0x4000_0040u32.to_le_bytes());
    header
}

/// Writes a number of four bytes into the buffer.
fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
