//! The rules that decide how a copy treats the files it copies.
//!
//! WHAT: the policy of a copy ([`CopyPolicy`]).
//! WHY: the two rules come from the security section of the specification
//!       (section 9.8): the verified copy compares the checksums, and a
//!       symbolic link is copied as a link and never dereferenced. The copy
//!       never decides them on its own — the composition root builds the
//!       policy from the rules that are in force, so that the same code
//!       behaves differently on a private machine and on a managed one. The
//!       difference is the setting, not the program.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::copy::CopyPolicy;
//!
//! let policy = CopyPolicy::strict();
//! assert!(policy.verifies_checksums());
//! assert!(!policy.follows_symlinks());
//! ```

/// The rules that decide how a copy treats the files it copies.
///
/// WHY: the two rules come from the security section of the specification
///       (section 9.8). The copy never decides them on its own: the
///       composition root builds the policy from the rules that are in force,
///       so that the same code behaves differently on a private machine and
///       on a managed one — the difference is the setting, not the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyPolicy {
    verify_checksums: bool,
    follow_symlinks: bool,
}

impl CopyPolicy {
    /// Builds a policy from the two rules.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyPolicy;
    ///
    /// let policy = CopyPolicy::new(false, false);
    /// assert!(!policy.verifies_checksums());
    /// assert!(!policy.follows_symlinks());
    /// ```
    pub const fn new(verify_checksums: bool, follow_symlinks: bool) -> Self {
        Self {
            verify_checksums,
            follow_symlinks,
        }
    }

    /// The policy of the specification: the copy is verified and a symbolic link
    /// is copied as a link.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyPolicy;
    ///
    /// let policy = CopyPolicy::strict();
    /// assert!(policy.verifies_checksums());
    /// assert!(!policy.follows_symlinks());
    /// ```
    pub const fn strict() -> Self {
        Self::new(true, false)
    }

    /// Whether the verified copy compares the checksums of source and
    /// destination.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyPolicy;
    ///
    /// assert!(CopyPolicy::strict().verifies_checksums());
    /// ```
    pub const fn verifies_checksums(&self) -> bool {
        self.verify_checksums
    }

    /// Whether a symbolic link is dereferenced during the copy.
    ///
    /// WHY: the switch names the deviation from the rule of the specification —
    ///       "a symbolic link is copied as a symbolic link and is never
    ///       dereferenced". The rule is in force exactly when this answer is
    ///       `false`.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyPolicy;
    ///
    /// assert!(!CopyPolicy::strict().follows_symlinks());
    /// ```
    pub const fn follows_symlinks(&self) -> bool {
        self.follow_symlinks
    }
}
