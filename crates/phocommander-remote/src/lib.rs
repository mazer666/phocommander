//! The network sources of Phocommander.
//!
//! WHAT: the crate that offers directories which live on another machine —
//!       SFTP, FTP, SMB, WebDAV and S3-compatible object storage.
//! WHY: a remote source is not a second kind of file manager. It is another
//!       source of the same "content of a directory", so copying, moving,
//!       renaming, deleting, viewing and searching work exactly as they do
//!       locally. The core stays free of protocol details, and the trust
//!       boundary stays in one place: everything a server sends is untrusted
//!       input, from the entry names to the sizes.
//! BOUNDARY: this crate speaks the protocols. It draws nothing, it writes no
//!       local configuration and it stores no credential of its own — the
//!       credential belongs into the keyring of the operating system.
//!
//! MILESTONE: the crate is created in M0 so that the boundary exists from the
//! first day. SMB and WebDAV arrive in M6, SFTP and FTP together with the
//! connection management, the keyring integration and the hardening of the
//! network attack surface. S3 and OAuth arrive in M7.
//!
//! EXAMPLE:
//! ```no_run
//! // Milestone M6 will look like this:
//! // let entries = phocommander_remote::list(&connection)?;
//! ```
