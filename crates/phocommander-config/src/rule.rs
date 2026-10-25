//! The security rules of the product.
//!
//! WHAT: the identifier of a rule ([`SecurityRuleId`]), the state of a rule
//!       ([`SecurityRuleState`]), the place a state comes from
//!       ([`SecurityRuleSource`]) and a rule with both ([`SecurityRule`]).
//! WHY: a switch in the configuration file is not a rule. The user has to be
//!       able to read *what* the application does, not only *which* switch is
//!       set: every rule therefore carries a title, an explanation and the
//!       name of the switch that decides it. Three of the rules are
//!       guarantees that no switch can break (NFR-32), and they say so
//!       themselves.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::{SecurityRuleId, SecurityRuleState};
//!
//! let rule = SecurityRuleId::ChecksumOnCopy;
//! assert!(rule.is_switchable());
//! assert_eq!(rule.switch_name(), Some("checksum_on_copy"));
//! assert!(SecurityRuleState::Enabled.is_enabled());
//! ```

/// The identifier of a security rule.
///
/// WHY: the rule is the unit that the settings surface shows and that the
/// log names. The identifier is stable, so that a translation of the rule and
/// a stored preference can refer to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecurityRuleId {
    /// The application sends nothing anywhere.
    NoTelemetry,

    /// The application never asks for administrator rights.
    NoElevation,

    /// The application keeps no index and never reads a file that the user
    /// has not opened.
    NoBackgroundIndex,

    /// The viewer reads a file only when the user opens it.
    ReadOnlyOnDemand,

    /// The content of a directory is read again before it is shown.
    VerifyDirectoryBeforeRefresh,

    /// The verified copy compares the checksums of source and destination.
    ChecksumOnCopy,

    /// A symbolic link is copied as a symbolic link and never dereferenced.
    SymlinksAreNeverFollowed,

    /// The application leaves no state of the session behind.
    LeavesNothingBehind,
}

impl SecurityRuleId {
    /// The name of the rule, as the log and the settings write it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleId;
    ///
    /// assert_eq!(SecurityRuleId::ChecksumOnCopy.as_str(), "checksum_on_copy");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            SecurityRuleId::NoTelemetry => "no_telemetry",
            SecurityRuleId::NoElevation => "no_elevation",
            SecurityRuleId::NoBackgroundIndex => "no_background_index",
            SecurityRuleId::ReadOnlyOnDemand => "read_only_on_demand",
            SecurityRuleId::VerifyDirectoryBeforeRefresh => "verify_directory_before_refresh",
            SecurityRuleId::ChecksumOnCopy => "checksum_on_copy",
            SecurityRuleId::SymlinksAreNeverFollowed => "symlinks_are_never_followed",
            SecurityRuleId::LeavesNothingBehind => "leaves_nothing_behind",
        }
    }

    /// The title of the rule in the settings surface.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleId;
    ///
    /// assert!(!SecurityRuleId::NoTelemetry.title().is_empty());
    /// ```
    pub const fn title(&self) -> &'static str {
        match self {
            SecurityRuleId::NoTelemetry => "No telemetry",
            SecurityRuleId::NoElevation => "No elevation",
            SecurityRuleId::NoBackgroundIndex => "No background index",
            SecurityRuleId::ReadOnlyOnDemand => "Read only on demand",
            SecurityRuleId::VerifyDirectoryBeforeRefresh => "Verify directory before refresh",
            SecurityRuleId::ChecksumOnCopy => "Checksum on copy",
            SecurityRuleId::SymlinksAreNeverFollowed => "Symbolic links are never followed",
            SecurityRuleId::LeavesNothingBehind => "Leaves nothing behind",
        }
    }

    /// The explanation of the rule, one sentence for the settings surface.
    ///
    /// WHY: the rule has to be visible and not only switched. A user who
    /// reads "the application compares the checksums of a copy" understands
    /// what the switch does; a user who reads "checksum_on_copy = true" does
    /// not.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleId;
    ///
    /// assert!(SecurityRuleId::NoElevation.explanation().contains("administrator"));
    /// ```
    pub const fn explanation(&self) -> &'static str {
        match self {
            SecurityRuleId::NoTelemetry => {
                "The application sends nothing anywhere and contains no update check."
            }
            SecurityRuleId::NoElevation => {
                "The application never asks for administrator rights and never restarts itself as administrator."
            }
            SecurityRuleId::NoBackgroundIndex => {
                "The application keeps no index and never reads a file that the user has not opened."
            }
            SecurityRuleId::ReadOnlyOnDemand => {
                "The viewer reads a file only when the user opens it in the viewer."
            }
            SecurityRuleId::VerifyDirectoryBeforeRefresh => {
                "The content of a directory is read again before it is shown, so that a change made elsewhere is noticed."
            }
            SecurityRuleId::ChecksumOnCopy => {
                "The verified copy compares the checksums of source and destination."
            }
            SecurityRuleId::SymlinksAreNeverFollowed => {
                "A symbolic link is copied as a symbolic link and is never dereferenced."
            }
            SecurityRuleId::LeavesNothingBehind => {
                "The application writes no state of the session and leaves nothing behind on the machine."
            }
        }
    }

    /// Whether the rule can be switched.
    ///
    /// WHY: three of the rules are guarantees of the product and cannot be
    /// switched off (NFR-32). The settings surface shows them as fixed, so
    /// that the user knows that they are not missing from the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleId;
    ///
    /// assert!(!SecurityRuleId::NoTelemetry.is_switchable());
    /// assert!(SecurityRuleId::ChecksumOnCopy.is_switchable());
    /// ```
    pub const fn is_switchable(&self) -> bool {
        !matches!(
            self,
            SecurityRuleId::NoTelemetry
                | SecurityRuleId::NoElevation
                | SecurityRuleId::NoBackgroundIndex
        )
    }

    /// The switch of the configuration file that decides the rule.
    ///
    /// WHY: two of the switches are named after the deviation from the rule
    /// and not after the rule itself: `follow_symlinks` decides whether a
    /// link is followed, and `remember_session` decides whether the session
    /// is remembered. The name of the switch is written down here, so that
    /// the settings surface can put the switch and the rule next to each
    /// other without guessing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleId;
    ///
    /// assert_eq!(
    ///     SecurityRuleId::ChecksumOnCopy.switch_name(),
    ///     Some("checksum_on_copy")
    /// );
    /// assert_eq!(SecurityRuleId::NoTelemetry.switch_name(), None);
    /// ```
    pub const fn switch_name(&self) -> Option<&'static str> {
        match self {
            SecurityRuleId::NoTelemetry
            | SecurityRuleId::NoElevation
            | SecurityRuleId::NoBackgroundIndex => None,
            SecurityRuleId::ReadOnlyOnDemand => Some("read_only_on_demand"),
            SecurityRuleId::VerifyDirectoryBeforeRefresh => Some("verify_directory_before_refresh"),
            SecurityRuleId::ChecksumOnCopy => Some("checksum_on_copy"),
            SecurityRuleId::SymlinksAreNeverFollowed => Some("follow_symlinks"),
            SecurityRuleId::LeavesNothingBehind => Some("remember_session"),
        }
    }
}

/// The state of a security rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityRuleState {
    /// The rule is in force.
    Enabled,

    /// The rule is not in force.
    Disabled,
}

impl SecurityRuleState {
    /// Whether the rule is in force.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleState;
    ///
    /// assert!(SecurityRuleState::Enabled.is_enabled());
    /// assert!(!SecurityRuleState::Disabled.is_enabled());
    /// ```
    pub const fn is_enabled(&self) -> bool {
        matches!(self, SecurityRuleState::Enabled)
    }
}

/// Where the state of a rule comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityRuleSource {
    /// The rule is a guarantee of the product and cannot be switched off.
    Guarantee,

    /// The profile of the configuration file decided the state of the rule.
    Profile(crate::schema::SecurityProfile),

    /// The switch in the configuration file decided the state of the rule.
    Switch(&'static str),
}

impl SecurityRuleSource {
    /// The name of the source, as the log writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleSource;
    ///
    /// let source = SecurityRuleSource::Switch("checksum_on_copy");
    /// assert_eq!(source.as_str(), "switch");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            SecurityRuleSource::Guarantee => "guarantee",
            SecurityRuleSource::Profile(_) => "profile",
            SecurityRuleSource::Switch(_) => "switch",
        }
    }
}

/// One security rule in the state that is really in force.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityRule {
    /// Which rule this is.
    pub id: SecurityRuleId,

    /// Whether the rule is in force.
    pub state: SecurityRuleState,

    /// Where the state comes from.
    pub source: SecurityRuleSource,
}

impl SecurityRule {
    /// Whether the rule is in force.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::{SecurityRule, SecurityRuleId, SecurityRuleState, SecurityRuleSource};
    ///
    /// let rule = SecurityRule {
    ///     id: SecurityRuleId::ChecksumOnCopy,
    ///     state: SecurityRuleState::Enabled,
    ///     source: SecurityRuleSource::Switch("checksum_on_copy"),
    /// };
    /// assert!(rule.is_enabled());
    /// ```
    pub const fn is_enabled(&self) -> bool {
        self.state.is_enabled()
    }

    /// The title of the rule in the settings surface.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleId;
    ///
    /// assert_eq!(SecurityRuleId::NoTelemetry.title(), "No telemetry");
    /// ```
    pub fn title(&self) -> &'static str {
        self.id.title()
    }

    /// The explanation of the rule in the settings surface.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SecurityRuleId;
    ///
    /// assert!(SecurityRuleId::ChecksumOnCopy.explanation().contains("checksums"));
    /// ```
    pub fn explanation(&self) -> &'static str {
        self.id.explanation()
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;
