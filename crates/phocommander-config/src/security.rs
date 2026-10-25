//! The security rules that are really in force.
//!
//! WHAT: the resolution of the {[security]} section of the configuration file
//!       into the set of rules that applies ([`EffectiveSecurity`]) and the
//!       function that performs the resolution ([`resolve_security`]).
//! WHY: the specification makes the security rules of the product
//!       configurable (sections 9.7 and 9.8) and keeps three of them as
//!       guarantees that no switch can break. The resolution is a pure
//!       function — the same profile and the same switches always produce the
//!       same rules — so that the settings surface, the log and the tests
//!       cannot disagree about what the application does.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::{SecurityProfile, SecurityRuleId, resolve_security};
//! use phocommander_config::schema::Security;
//!
//! let settings = Security {
//!     profile: SecurityProfile::Corporate,
//!     ..Security::default()
//! };
//! let rules = resolve_security(&settings);
//! assert!(rules.is_enabled(SecurityRuleId::ChecksumOnCopy));
//! assert!(rules.is_enabled(SecurityRuleId::NoBackgroundIndex));
//! ```

use crate::notice::ConfigNotice;
use crate::rule::{SecurityRule, SecurityRuleId, SecurityRuleSource, SecurityRuleState};
use crate::schema::{Security, SecurityProfile};

/// The default value of the switch `verify_directory_before_refresh`.
///
/// WHY: the value is written down once, so that the default of the file, the
/// baseline of the profile `standard` and the shipped configuration file
/// cannot drift apart.
pub const DEFAULT_VERIFY_DIRECTORY_BEFORE_REFRESH: bool = true;

/// The default value of the switch `background_index`.
///
/// WHY: the application keeps no index. The switch exists so that the value
/// is visible; the guarantee itself does not depend on it.
pub const DEFAULT_BACKGROUND_INDEX: bool = false;

/// The default value of the switch `remember_session`.
pub const DEFAULT_REMEMBER_SESSION: bool = true;

/// The default value of the switch `read_only_on_demand`.
pub const DEFAULT_READ_ONLY_ON_DEMAND: bool = true;

/// The default value of the switch `checksum_on_copy`.
pub const DEFAULT_CHECKSUM_ON_COPY: bool = true;

/// The default value of the switch `follow_symlinks`.
///
/// WHY: the value is `false`, because a symbolic link is copied as a
/// symbolic link and never dereferenced. The name of the switch names the
/// deviation from that rule, not the rule itself.
pub const DEFAULT_FOLLOW_SYMLINKS: bool = false;

/// The values of the switches of the configuration file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Switches {
    verify_directory_before_refresh: bool,
    remember_session: bool,
    read_only_on_demand: bool,
    checksum_on_copy: bool,
    follow_symlinks: bool,
}

impl Switches {
    /// The values of the profile `standard`.
    ///
    /// WHY: the profile is the suggestion of the specification: the rules of
    /// section 9.7 and the convenience switches at their default. The
    /// session is remembered, because remembering it weakens no guarantee of
    /// section 9.7 — the file it produces stays inside the configuration
    /// directory.
    const fn standard() -> Self {
        Self {
            verify_directory_before_refresh: DEFAULT_VERIFY_DIRECTORY_BEFORE_REFRESH,
            remember_session: DEFAULT_REMEMBER_SESSION,
            read_only_on_demand: DEFAULT_READ_ONLY_ON_DEMAND,
            checksum_on_copy: DEFAULT_CHECKSUM_ON_COPY,
            follow_symlinks: DEFAULT_FOLLOW_SYMLINKS,
        }
    }

    /// The values of the profile `corporate`.
    ///
    /// WHY: the profile enables every rule of section 9.7 and gives up the
    /// conveniences that cost something: the session is not remembered, so
    /// that a managed machine keeps no trace of the application. Every other
    /// rule is stronger than in the profile `standard`, never weaker.
    const fn corporate() -> Self {
        Self {
            remember_session: false,
            ..Self::standard()
        }
    }

    /// The values that the file itself carries.
    const fn from_settings(settings: &Security) -> Self {
        Self {
            verify_directory_before_refresh: settings.verify_directory_before_refresh,
            remember_session: settings.remember_session,
            read_only_on_demand: settings.read_only_on_demand,
            checksum_on_copy: settings.checksum_on_copy,
            follow_symlinks: settings.follow_symlinks,
        }
    }
}

/// The security rules that are really in force.
///
/// WHY: the type is the answer to the question "what does the application do
/// on this machine". It carries the rules with their state and their
/// explanation, so that the settings surface can show them, and it carries
/// the messages about the settings that had no effect, so that the log can
/// report them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveSecurity {
    profile: SecurityProfile,
    rules: Vec<SecurityRule>,
    notices: Vec<ConfigNotice>,
    switches: Switches,
}

impl EffectiveSecurity {
    /// The rules that are in force on a machine whose configuration file
    /// could not be used.
    ///
    /// WHY: a broken file must not weaken the application. The safe set is
    /// the profile `corporate`, because it is the set in which every rule of
    /// section 9.7 is in force.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::{EffectiveSecurity, SecurityRuleId};
    ///
    /// let rules = EffectiveSecurity::safe();
    /// assert!(rules.is_enabled(SecurityRuleId::ChecksumOnCopy));
    /// assert!(rules.is_enabled(SecurityRuleId::LeavesNothingBehind));
    /// ```
    pub fn safe() -> Self {
        resolve_security(&Security::safe())
    }

    /// The profile that decided the rules.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::{EffectiveSecurity, SecurityProfile};
    ///
    /// assert_eq!(EffectiveSecurity::safe().profile(), SecurityProfile::Corporate);
    /// ```
    pub fn profile(&self) -> SecurityProfile {
        self.profile
    }

    /// The rules that are in force, in a fixed order.
    ///
    /// WHY: the order is fixed, so that the settings surface and the log
    /// show the rules in the same order every time.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// let rules = EffectiveSecurity::safe();
    /// assert_eq!(rules.rules().len(), 8);
    /// ```
    pub fn rules(&self) -> &[SecurityRule] {
        &self.rules
    }

    /// One rule, if the set carries it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::{EffectiveSecurity, SecurityRuleId};
    ///
    /// let rules = EffectiveSecurity::safe();
    /// assert!(rules.rule(SecurityRuleId::NoTelemetry).is_some());
    /// ```
    pub fn rule(&self, id: SecurityRuleId) -> Option<&SecurityRule> {
        self.rules.iter().find(|rule| rule.id == id)
    }

    /// Whether a rule is in force.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::{EffectiveSecurity, SecurityRuleId};
    ///
    /// let rules = EffectiveSecurity::safe();
    /// assert!(rules.is_enabled(SecurityRuleId::ReadOnlyOnDemand));
    /// ```
    pub fn is_enabled(&self, id: SecurityRuleId) -> bool {
        self.rule(id).map(SecurityRule::is_enabled).unwrap_or(false)
    }

    /// The messages about the settings that had no effect.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(EffectiveSecurity::safe().notices().is_empty());
    /// ```
    pub fn notices(&self) -> &[ConfigNotice] {
        &self.notices
    }

    /// Whether the content of a directory is read again before it is shown.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(EffectiveSecurity::safe().verifies_directory_before_refresh());
    /// ```
    pub fn verifies_directory_before_refresh(&self) -> bool {
        self.switches.verify_directory_before_refresh
    }

    /// Whether the state of the session is written at the end.
    ///
    /// WHY: the switch is named after what it writes. The rule that is
    /// shown to the user is the opposite one — "leaves nothing behind" — and
    /// it is in force exactly when this answer is `false`.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(!EffectiveSecurity::safe().remembers_session());
    /// ```
    pub fn remembers_session(&self) -> bool {
        self.switches.remember_session
    }

    /// Whether the viewer reads a file only when the user opens it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(EffectiveSecurity::safe().reads_only_on_demand());
    /// ```
    pub fn reads_only_on_demand(&self) -> bool {
        self.switches.read_only_on_demand
    }

    /// Whether the verified copy compares the checksums.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(EffectiveSecurity::safe().verifies_checksums());
    /// ```
    pub fn verifies_checksums(&self) -> bool {
        self.switches.checksum_on_copy
    }

    /// Whether a symbolic link is dereferenced during a copy.
    ///
    /// WHY: the switch names the deviation from the rule. The rule itself —
    /// "a symbolic link is copied as a symbolic link" — is in force exactly
    /// when this answer is `false`.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(!EffectiveSecurity::safe().follows_symlinks());
    /// ```
    pub fn follows_symlinks(&self) -> bool {
        self.switches.follow_symlinks
    }

    /// Whether the application keeps a background index.
    ///
    /// WHY: the answer is always `false`. The guarantee does not depend on a
    /// switch, and this function is the place where that is written down.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(!EffectiveSecurity::safe().keeps_background_index());
    /// ```
    pub fn keeps_background_index(&self) -> bool {
        false
    }

    /// Whether the application sends telemetry.
    ///
    /// WHY: the answer is always `false`. There is nothing to switch off,
    /// because there is nothing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(!EffectiveSecurity::safe().sends_telemetry());
    /// ```
    pub fn sends_telemetry(&self) -> bool {
        false
    }

    /// Whether the application asks for administrator rights.
    ///
    /// WHY: the answer is always `false`. A file manager that needs
    /// elevation for a single path has to be run elevated for everything.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::EffectiveSecurity;
    ///
    /// assert!(!EffectiveSecurity::safe().requests_elevation());
    /// ```
    pub fn requests_elevation(&self) -> bool {
        false
    }
}

/// Resolves the settings of the file into the rules that are in force.
///
/// WHY: the resolution is a pure function. The same settings always produce
/// the same rules and the same messages, so that the settings surface, the
/// log and the tests cannot disagree about what the application does. The
/// profile decides: in the profiles `standard` and `corporate` the switches
/// of the file are read but not obeyed, and every difference is reported as
/// a message; in the profile `custom` the switches of the file are the
/// rules.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::{SecurityProfile, SecurityRuleId, resolve_security};
/// use phocommander_config::schema::Security;
///
/// let settings = Security {
///     profile: SecurityProfile::Custom,
///     checksum_on_copy: false,
///     ..Security::default()
/// };
/// let rules = resolve_security(&settings);
/// assert!(!rules.is_enabled(SecurityRuleId::ChecksumOnCopy));
/// ```
pub fn resolve_security(settings: &Security) -> EffectiveSecurity {
    let mut notices = Vec::new();
    collect_guarantee_notices(settings, &mut notices);

    let switches = match settings.profile {
        SecurityProfile::Standard => Switches::standard(),
        SecurityProfile::Corporate => Switches::corporate(),
        SecurityProfile::Custom => Switches::from_settings(settings),
    };

    if settings.profile != SecurityProfile::Custom {
        collect_profile_notices(settings.profile, settings, &switches, &mut notices);
    }

    EffectiveSecurity {
        profile: settings.profile,
        rules: build_rules(settings.profile, &switches),
        notices,
        switches,
    }
}

/// Reports the settings that try to break a guarantee.
fn collect_guarantee_notices(settings: &Security, notices: &mut Vec<ConfigNotice>) {
    if settings.telemetry {
        notices.push(ConfigNotice::warning(
            "security.telemetry",
            "the application has no telemetry; there is nothing to switch off, so the value was ignored",
        ));
    }

    if settings.elevation {
        notices.push(ConfigNotice::warning(
            "security.elevation",
            "the application never asks for administrator rights, so the value was ignored",
        ));
    }

    if settings.background_index {
        notices.push(ConfigNotice::warning(
            "security.background_index",
            "the application keeps no index and never reads a file that the user has not opened, so the value was ignored",
        ));
    }
}

/// Reports the switches that the profile overrules.
fn collect_profile_notices(
    profile: SecurityProfile,
    settings: &Security,
    effective: &Switches,
    notices: &mut Vec<ConfigNotice>,
) {
    let mut report = |field: &'static str, file_value: bool, effective_value: bool| {
        if file_value == effective_value {
            return;
        }
        notices.push(ConfigNotice::new(
            field,
            format!(
                "the profile `{}` sets this rule to {effective_value}; the file says {file_value}. \
                 The profile `custom` follows the value of the file.",
                profile.as_str()
            ),
        ));
    };

    report(
        "security.verify_directory_before_refresh",
        settings.verify_directory_before_refresh,
        effective.verify_directory_before_refresh,
    );
    report(
        "security.remember_session",
        settings.remember_session,
        effective.remember_session,
    );
    report(
        "security.read_only_on_demand",
        settings.read_only_on_demand,
        effective.read_only_on_demand,
    );
    report(
        "security.checksum_on_copy",
        settings.checksum_on_copy,
        effective.checksum_on_copy,
    );
    report(
        "security.follow_symlinks",
        settings.follow_symlinks,
        effective.follow_symlinks,
    );
}

/// Builds the rules that the switches describe.
fn build_rules(profile: SecurityProfile, switches: &Switches) -> Vec<SecurityRule> {
    let source = |switch: &'static str| match profile {
        SecurityProfile::Custom => SecurityRuleSource::Switch(switch),
        _ => SecurityRuleSource::Profile(profile),
    };

    vec![
        SecurityRule {
            id: SecurityRuleId::NoTelemetry,
            state: SecurityRuleState::Enabled,
            source: SecurityRuleSource::Guarantee,
        },
        SecurityRule {
            id: SecurityRuleId::NoElevation,
            state: SecurityRuleState::Enabled,
            source: SecurityRuleSource::Guarantee,
        },
        SecurityRule {
            id: SecurityRuleId::NoBackgroundIndex,
            state: SecurityRuleState::Enabled,
            source: SecurityRuleSource::Guarantee,
        },
        SecurityRule {
            id: SecurityRuleId::ReadOnlyOnDemand,
            state: state_of(switches.read_only_on_demand),
            source: source("read_only_on_demand"),
        },
        SecurityRule {
            id: SecurityRuleId::VerifyDirectoryBeforeRefresh,
            state: state_of(switches.verify_directory_before_refresh),
            source: source("verify_directory_before_refresh"),
        },
        SecurityRule {
            id: SecurityRuleId::ChecksumOnCopy,
            state: state_of(switches.checksum_on_copy),
            source: source("checksum_on_copy"),
        },
        SecurityRule {
            id: SecurityRuleId::SymlinksAreNeverFollowed,
            state: state_of(!switches.follow_symlinks),
            source: source("follow_symlinks"),
        },
        SecurityRule {
            id: SecurityRuleId::LeavesNothingBehind,
            state: state_of(!switches.remember_session),
            source: source("remember_session"),
        },
    ]
}

/// Turns a value into the state of a rule.
const fn state_of(value: bool) -> SecurityRuleState {
    if value {
        SecurityRuleState::Enabled
    } else {
        SecurityRuleState::Disabled
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;
