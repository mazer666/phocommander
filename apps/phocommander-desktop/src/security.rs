//! The translation of the configuration into the rules of the subsystems.
//!
//! WHAT: the plan that carries the rules of the configuration into the
//!       subsystems ([`SecurityPlan`]).
//! WHY: the workspace is flat, so no crate below `crates/` may depend on a
//!       sibling. The configuration crate therefore cannot hand its rules to
//!       the operations crate, and the operations crate cannot ask the
//!       configuration crate what it should do. The composition root is the
//!       only place that may see both, and it is the place where the rules
//!       become the values the subsystems understand (ADR-0001, option B).
//!       The translation is written down once here, so that the operations,
//!       the viewer and the interface cannot disagree about what the
//!       configuration means.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::EffectiveSecurity;
//! use phocommander_desktop::security::SecurityPlan;
//!
//! let rules = EffectiveSecurity::safe();
//! let plan = SecurityPlan::from_effective(&rules);
//! assert!(plan.copy_policy().verifies_checksums());
//! assert!(!plan.copy_policy().follows_symlinks());
//! ```

use phocommander_config::EffectiveSecurity;
use phocommander_ops::copy::CopyPolicy;
use phocommander_ui::refresh::RefreshPolicy;
use phocommander_viewer::policy::ReadPolicy;

/// The rules of the configuration, translated into the values that the
/// subsystems understand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPlan {
    copy: CopyPolicy,
    read: ReadPolicy,
    refresh: RefreshPolicy,
}

impl SecurityPlan {
    /// Builds the plan from the rules that are in force.
    ///
    /// WHY: the translation is a pure function of the rules. The same rules
    /// always produce the same plan, so that the application cannot behave
    /// differently from one start to the next.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    /// use phocommander_desktop::security::SecurityPlan;
    ///
    /// let plan = SecurityPlan::from_effective(&EffectiveSecurity::safe());
    /// assert!(plan.read_policy().reads_only_on_demand());
    /// assert!(plan.refresh_policy().verifies_before_refresh());
    /// ```
    pub fn from_effective(effective: &EffectiveSecurity) -> Self {
        Self {
            copy: CopyPolicy::new(effective.verifies_checksums(), effective.follows_symlinks()),
            read: ReadPolicy::new(effective.reads_only_on_demand()),
            refresh: RefreshPolicy::new(effective.verifies_directory_before_refresh()),
        }
    }

    /// The rules that decide how a copy treats the files it copies.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    /// use phocommander_desktop::security::SecurityPlan;
    ///
    /// let plan = SecurityPlan::from_effective(&EffectiveSecurity::safe());
    /// assert!(plan.copy_policy().verifies_checksums());
    /// ```
    pub fn copy_policy(&self) -> &CopyPolicy {
        &self.copy
    }

    /// The rules that decide what the viewer may read.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    /// use phocommander_desktop::security::SecurityPlan;
    ///
    /// let plan = SecurityPlan::from_effective(&EffectiveSecurity::safe());
    /// assert!(plan.read_policy().reads_only_on_demand());
    /// ```
    pub fn read_policy(&self) -> &ReadPolicy {
        &self.read
    }

    /// The rule that decides whether a directory is read again before it is
    /// shown.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    /// use phocommander_desktop::security::SecurityPlan;
    ///
    /// let plan = SecurityPlan::from_effective(&EffectiveSecurity::safe());
    /// assert!(plan.refresh_policy().verifies_before_refresh());
    /// ```
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use phocommander_config::schema::Security;
    use phocommander_config::{SecurityProfile, resolve_security};

    #[test]
    fn the_safe_rules_give_the_strict_plan() {
        // arrange
        let rules = EffectiveSecurity::safe();

        // act
        let plan = SecurityPlan::from_effective(&rules);

        // assert
        assert!(plan.copy_policy().verifies_checksums());
        assert!(!plan.copy_policy().follows_symlinks());
        assert!(plan.read_policy().reads_only_on_demand());
        assert!(plan.refresh_policy().verifies_before_refresh());
    }

    #[test]
    fn a_relaxed_rule_reaches_the_copy() {
        // arrange
        let rules = resolve_security(&Security {
            profile: SecurityProfile::Custom,
            checksum_on_copy: false,
            follow_symlinks: true,
            ..Security::default()
        });

        // act
        let plan = SecurityPlan::from_effective(&rules);

        // assert
        assert!(!plan.copy_policy().verifies_checksums());
        assert!(plan.copy_policy().follows_symlinks());
    }

    #[test]
    fn a_relaxed_rule_reaches_the_viewer() {
        // arrange
        let rules = resolve_security(&Security {
            profile: SecurityProfile::Custom,
            read_only_on_demand: false,
            ..Security::default()
        });

        // act
        let plan = SecurityPlan::from_effective(&rules);

        // assert
        assert!(!plan.read_policy().reads_only_on_demand());
    }

    #[test]
    fn a_relaxed_rule_reaches_the_interface() {
        // arrange
        let rules = resolve_security(&Security {
            profile: SecurityProfile::Custom,
            verify_directory_before_refresh: false,
            ..Security::default()
        });

        // act
        let plan = SecurityPlan::from_effective(&rules);

        // assert
        assert!(!plan.refresh_policy().verifies_before_refresh());
    }
}
