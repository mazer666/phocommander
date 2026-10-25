//! The rule that decides whether a directory is read again before it is shown.
//!
//! WHAT: the rule of the security section of the specification that decides
//!       whether the content of a directory is read again before the panel
//!       shows it ([`RefreshPolicy`]).
//! WHY: a change that was made elsewhere is noticed only if the directory is
//!       read again. On a network drive that costs round trips, which is why
//!       the specification makes it a switch (section 9.8). The rule belongs
//!       to the interface, because the interface knows when a panel is shown
//!       again; the value comes from the configuration, and the composition
//!       root hands it over.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::refresh::RefreshPolicy;
//!
//! let policy = RefreshPolicy::strict();
//! assert!(policy.verifies_before_refresh());
//! ```

/// The rule that decides whether a directory is read again before it is
/// shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshPolicy {
    verify_before_refresh: bool,
}

impl RefreshPolicy {
    /// Builds the rule.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::refresh::RefreshPolicy;
    ///
    /// let policy = RefreshPolicy::new(false);
    /// assert!(!policy.verifies_before_refresh());
    /// ```
    pub const fn new(verify_before_refresh: bool) -> Self {
        Self {
            verify_before_refresh,
        }
    }

    /// The rule of the specification: the directory is read again before it
    /// is shown.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::refresh::RefreshPolicy;
    ///
    /// assert!(RefreshPolicy::strict().verifies_before_refresh());
    /// ```
    pub const fn strict() -> Self {
        Self::new(true)
    }

    /// Whether the directory is read again before it is shown.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::refresh::RefreshPolicy;
    ///
    /// assert!(RefreshPolicy::strict().verifies_before_refresh());
    /// ```
    pub const fn verifies_before_refresh(&self) -> bool {
        self.verify_before_refresh
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_strict_rule_reads_the_directory_again() {
        // act
        let policy = RefreshPolicy::strict();

        // assert
        assert!(policy.verifies_before_refresh());
    }

    #[test]
    fn the_rule_can_be_switched_off() {
        // act
        let policy = RefreshPolicy::new(false);

        // assert
        assert!(!policy.verifies_before_refresh());
    }
}
