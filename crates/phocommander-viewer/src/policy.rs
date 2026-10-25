//! The rules that decide what the viewer may read.
//!
//! WHAT: the trigger of a read ([`ReadTrigger`]) and the policy that decides
//!       whether the trigger may read a file ([`ReadPolicy`]).
//! WHY: the specification (section 9.7) demands that the application reads
//!       only what the user asked for: it does not index the disk, does not
//!       preview directories in advance and does not classify files by their
//!       content. A file is read because somebody asked for it, and the
//!       question "who asked" is answered by the trigger. The trigger
//!       `BackgroundScan` is refused by every policy — that guarantee is
//!       written into the code and not into a switch, because a switch that
//!       could be moved would not be a guarantee.
//! EXAMPLE:
//! ```no_run
//! use phocommander_viewer::policy::{ReadPolicy, ReadTrigger};
//!
//! let policy = ReadPolicy::strict();
//! assert!(policy.allows(ReadTrigger::UserCommand));
//! assert!(!policy.allows(ReadTrigger::SelectionPreview));
//! assert!(!policy.allows(ReadTrigger::BackgroundScan));
//! ```

/// What caused a file to be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadTrigger {
    /// The user opened the file in the viewer.
    ///
    /// WHY: this is the only trigger that every policy allows. The user
    /// asked for the content of this file and nothing else.
    UserCommand,

    /// The interface wants to show a preview of the marked entry.
    ///
    /// WHY: a preview reads a file the user has not opened. The rule
    /// "read_only_on_demand" decides whether that may happen.
    SelectionPreview,

    /// Something wants to read many files at once, for example an index.
    ///
    /// WHY: the application keeps no index. The trigger exists so that the
    /// attempt is refused by name and not only by accident.
    BackgroundScan,
}

/// The rules that decide what the viewer may read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadPolicy {
    read_on_demand: bool,
}

impl ReadPolicy {
    /// Builds a policy.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    ///
    /// let policy = ReadPolicy::new(false);
    /// assert!(!policy.reads_only_on_demand());
    /// ```
    pub const fn new(read_on_demand: bool) -> Self {
        Self { read_on_demand }
    }

    /// The policy of the specification: a file is read only when the user
    /// opens it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    ///
    /// assert!(ReadPolicy::strict().reads_only_on_demand());
    /// ```
    pub const fn strict() -> Self {
        Self::new(true)
    }

    /// Whether a file is read only when the user opens it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    ///
    /// assert!(ReadPolicy::strict().reads_only_on_demand());
    /// ```
    pub const fn reads_only_on_demand(&self) -> bool {
        self.read_on_demand
    }

    /// Whether the trigger may read a file.
    ///
    /// WHY: the background scan is refused by every policy. The guarantee
    /// "no background index" does not depend on a setting, and a setting
    /// that could be moved would not be a guarantee.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::{ReadPolicy, ReadTrigger};
    ///
    /// let relaxed = ReadPolicy::new(false);
    /// assert!(relaxed.allows(ReadTrigger::SelectionPreview));
    /// assert!(!relaxed.allows(ReadTrigger::BackgroundScan));
    /// ```
    pub const fn allows(&self, trigger: ReadTrigger) -> bool {
        match trigger {
            ReadTrigger::UserCommand => true,
            ReadTrigger::SelectionPreview => !self.read_on_demand,
            ReadTrigger::BackgroundScan => false,
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_strict_policy_reads_only_what_the_user_asked_for() {
        // arrange
        let policy = ReadPolicy::strict();

        // act, assert
        assert!(policy.reads_only_on_demand());
        assert!(policy.allows(ReadTrigger::UserCommand));
        assert!(!policy.allows(ReadTrigger::SelectionPreview));
    }

    #[test]
    fn a_relaxed_policy_allows_the_preview_of_a_marked_entry() {
        // arrange
        let policy = ReadPolicy::new(false);

        // act, assert
        assert!(!policy.reads_only_on_demand());
        assert!(policy.allows(ReadTrigger::UserCommand));
        assert!(policy.allows(ReadTrigger::SelectionPreview));
    }

    #[test]
    fn no_policy_ever_allows_a_background_scan() {
        // WHY: the guarantee "no background index" belongs to the code and
        // not to a switch.
        // arrange
        let policies = [ReadPolicy::strict(), ReadPolicy::new(false)];

        // act, assert
        for policy in policies {
            assert!(!policy.allows(ReadTrigger::BackgroundScan));
        }
    }
}
