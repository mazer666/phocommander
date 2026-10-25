//! The tests of the resolution of the security rules.
//!
//! WHAT: the tests of the three profiles, of the guarantees that no switch
//!       can break and of the messages that report a setting without effect.
//! WHY: the resolution decides what the application does on a machine. A
//!       profile that does not do what the specification says, or a switch
//!       that breaks a guarantee, would be a wrong application rather than a
//!       wrong display.
//! EXAMPLE: `cargo test -p phocommander-config security::`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::notice::NoticeSeverity;

/// Builds the settings of a file with the given profile.
fn settings(profile: SecurityProfile) -> Security {
    Security {
        profile,
        ..Security::default()
    }
}

#[test]
fn the_profile_corporate_enables_every_rule() {
    // arrange
    let settings = settings(SecurityProfile::Corporate);

    // act
    let effective = resolve_security(&settings);

    // assert
    for rule in effective.rules() {
        assert!(
            rule.is_enabled(),
            "the rule {} is not in force",
            rule.id.as_str()
        );
    }
    assert_eq!(effective.rules().len(), 8);
}

#[test]
fn the_profile_standard_keeps_the_convenience_switches_at_their_default() {
    // arrange
    let settings = settings(SecurityProfile::Standard);

    // act
    let effective = resolve_security(&settings);

    // assert
    assert!(effective.verifies_directory_before_refresh());
    assert!(effective.reads_only_on_demand());
    assert!(effective.verifies_checksums());
    assert!(!effective.follows_symlinks());
    assert!(effective.remembers_session());
}

#[test]
fn the_profile_standard_reports_the_switches_that_it_overrules() {
    // arrange
    let settings = Security {
        profile: SecurityProfile::Standard,
        remember_session: false,
        ..Security::default()
    };

    // act
    let effective = resolve_security(&settings);

    // assert
    assert!(effective.remembers_session());
    let notice = effective
        .notices()
        .iter()
        .find(|notice| notice.field == "security.remember_session");
    assert!(notice.is_some(), "{:?}", effective.notices());
}

#[test]
fn the_profile_custom_follows_the_switches_of_the_file() {
    // arrange
    let settings = Security {
        profile: SecurityProfile::Custom,
        verify_directory_before_refresh: false,
        remember_session: false,
        read_only_on_demand: false,
        checksum_on_copy: false,
        follow_symlinks: true,
        ..Security::default()
    };

    // act
    let effective = resolve_security(&settings);

    // assert
    assert!(!effective.verifies_directory_before_refresh());
    assert!(!effective.remembers_session());
    assert!(!effective.reads_only_on_demand());
    assert!(!effective.verifies_checksums());
    assert!(effective.follows_symlinks());
    assert!(effective.notices().is_empty());
}

#[test]
fn the_profile_custom_still_refuses_to_break_a_guarantee() {
    // arrange
    let settings = Security {
        profile: SecurityProfile::Custom,
        background_index: true,
        telemetry: true,
        elevation: true,
        ..Security::default()
    };

    // act
    let effective = resolve_security(&settings);

    // assert
    assert!(!effective.keeps_background_index());
    assert!(!effective.sends_telemetry());
    assert!(!effective.requests_elevation());
    assert_eq!(effective.notices().len(), 3);
    for notice in effective.notices() {
        assert_eq!(notice.severity, NoticeSeverity::Warning);
    }
}

#[test]
fn the_switch_of_the_background_index_has_no_effect() {
    // arrange
    let settings = Security {
        profile: SecurityProfile::Custom,
        background_index: true,
        ..Security::default()
    };

    // act
    let effective = resolve_security(&settings);

    // assert
    assert!(effective.is_enabled(SecurityRuleId::NoBackgroundIndex));
    let notice = effective
        .notices()
        .iter()
        .find(|notice| notice.field == "security.background_index");
    assert!(notice.is_some(), "{:?}", effective.notices());
}

#[test]
fn the_guarantees_are_in_force_in_every_profile() {
    // arrange
    let profiles = [
        SecurityProfile::Standard,
        SecurityProfile::Corporate,
        SecurityProfile::Custom,
    ];

    // act, assert
    for profile in profiles {
        let effective = resolve_security(&settings(profile));
        for id in [
            SecurityRuleId::NoTelemetry,
            SecurityRuleId::NoElevation,
            SecurityRuleId::NoBackgroundIndex,
        ] {
            assert!(effective.is_enabled(id), "{id:?} in {profile:?}");
            let source = effective.rule(id).map(|rule| rule.source);
            assert_eq!(source, Some(SecurityRuleSource::Guarantee));
            assert!(!id.is_switchable());
        }
    }
}

#[test]
fn the_safe_rules_are_the_rules_of_the_profile_corporate() {
    // arrange
    let safe = EffectiveSecurity::safe();

    // act
    let corporate = resolve_security(&settings(SecurityProfile::Corporate));

    // assert
    assert_eq!(safe.profile(), SecurityProfile::Corporate);
    assert_eq!(safe.rules(), corporate.rules());
    assert!(!safe.remembers_session());
}

#[test]
fn the_baseline_of_the_profile_standard_is_the_default_of_the_file() {
    // arrange
    let defaults = Security::default();

    // act
    let effective = resolve_security(&settings(SecurityProfile::Standard));

    // assert
    assert_eq!(
        effective.verifies_directory_before_refresh(),
        defaults.verify_directory_before_refresh
    );
    assert_eq!(effective.remembers_session(), defaults.remember_session);
    assert_eq!(
        effective.reads_only_on_demand(),
        defaults.read_only_on_demand
    );
    assert_eq!(effective.verifies_checksums(), defaults.checksum_on_copy);
    assert_eq!(effective.follows_symlinks(), defaults.follow_symlinks);
}

#[test]
fn the_resolution_is_the_same_every_time() {
    // arrange
    let settings = Security {
        profile: SecurityProfile::Custom,
        checksum_on_copy: false,
        follow_symlinks: true,
        ..Security::default()
    };

    // act
    let first = resolve_security(&settings);
    let second = resolve_security(&settings);

    // assert
    assert_eq!(first, second);
}

#[test]
fn the_rule_of_a_switch_that_is_inverted_reads_the_other_way_round() {
    // WHY: the switch `follow_symlinks` names the deviation from the rule
    // "a symbolic link is never followed". The rule is in force exactly when
    // the switch is off.
    // arrange
    let settings = Security {
        profile: SecurityProfile::Custom,
        follow_symlinks: true,
        ..Security::default()
    };

    // act
    let effective = resolve_security(&settings);

    // assert
    assert!(effective.follows_symlinks());
    assert!(!effective.is_enabled(SecurityRuleId::SymlinksAreNeverFollowed));
}
