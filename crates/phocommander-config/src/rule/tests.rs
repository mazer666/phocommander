//! The tests of the security rules.
//!
//! WHAT: the tests of the rule identifier, of the rule state and of the
//!       place a state comes from.
//! WHY: the rules are what the settings surface shows and what the log
//!       names. A rule without a title, without an explanation or without
//!       the name of its switch is a switch the user cannot understand.
//! EXAMPLE: `cargo test -p phocommander-config rule::`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::schema::SecurityProfile;

#[test]
fn every_rule_has_a_name_that_is_not_empty() {
    // act
    let names = [
        SecurityRuleId::NoTelemetry,
        SecurityRuleId::NoElevation,
        SecurityRuleId::NoBackgroundIndex,
        SecurityRuleId::ReadOnlyOnDemand,
        SecurityRuleId::VerifyDirectoryBeforeRefresh,
        SecurityRuleId::ChecksumOnCopy,
        SecurityRuleId::SymlinksAreNeverFollowed,
        SecurityRuleId::LeavesNothingBehind,
    ];

    // assert
    let mut written: Vec<&str> = names.iter().map(|id| id.as_str()).collect();
    written.sort_unstable();
    let mut unique = written.clone();
    unique.dedup();
    assert_eq!(written.len(), unique.len(), "two rules share a name");
    for name in written {
        assert!(!name.is_empty());
    }
}

#[test]
fn every_rule_carries_a_title_and_an_explanation() {
    // act
    let rules = [
        SecurityRuleId::NoTelemetry,
        SecurityRuleId::NoElevation,
        SecurityRuleId::NoBackgroundIndex,
        SecurityRuleId::ReadOnlyOnDemand,
        SecurityRuleId::VerifyDirectoryBeforeRefresh,
        SecurityRuleId::ChecksumOnCopy,
        SecurityRuleId::SymlinksAreNeverFollowed,
        SecurityRuleId::LeavesNothingBehind,
    ];

    // assert
    for rule in rules {
        assert!(!rule.title().is_empty(), "{}", rule.as_str());
        assert!(!rule.explanation().is_empty(), "{}", rule.as_str());
    }
}

#[test]
fn exactly_three_rules_cannot_be_switched() {
    // WHY: NFR-32 names the three guarantees: no telemetry, no elevation and
    // no background index. A fourth one would be a promise the product does
    // not make.
    // act
    let rules = [
        SecurityRuleId::NoTelemetry,
        SecurityRuleId::NoElevation,
        SecurityRuleId::NoBackgroundIndex,
        SecurityRuleId::ReadOnlyOnDemand,
        SecurityRuleId::VerifyDirectoryBeforeRefresh,
        SecurityRuleId::ChecksumOnCopy,
        SecurityRuleId::SymlinksAreNeverFollowed,
        SecurityRuleId::LeavesNothingBehind,
    ];

    // assert
    let fixed: Vec<&str> = rules
        .iter()
        .filter(|rule| !rule.is_switchable())
        .map(|rule| rule.as_str())
        .collect();
    assert_eq!(
        fixed,
        vec!["no_telemetry", "no_elevation", "no_background_index"]
    );
}

#[test]
fn a_switchable_rule_names_the_switch_that_decides_it() {
    // act
    let rules = [
        SecurityRuleId::ReadOnlyOnDemand,
        SecurityRuleId::VerifyDirectoryBeforeRefresh,
        SecurityRuleId::ChecksumOnCopy,
        SecurityRuleId::SymlinksAreNeverFollowed,
        SecurityRuleId::LeavesNothingBehind,
    ];

    // assert
    for rule in rules {
        assert!(rule.switch_name().is_some(), "{}", rule.as_str());
    }
    for rule in [
        SecurityRuleId::NoTelemetry,
        SecurityRuleId::NoElevation,
        SecurityRuleId::NoBackgroundIndex,
    ] {
        assert!(rule.switch_name().is_none(), "{}", rule.as_str());
    }
}

#[test]
fn the_state_of_a_rule_answers_the_question_it_is_asked() {
    // act, assert
    assert!(SecurityRuleState::Enabled.is_enabled());
    assert!(!SecurityRuleState::Disabled.is_enabled());
}

#[test]
fn the_source_of_a_state_has_a_name_for_the_log() {
    // act, assert
    assert_eq!(SecurityRuleSource::Guarantee.as_str(), "guarantee");
    assert_eq!(
        SecurityRuleSource::Profile(SecurityProfile::Corporate).as_str(),
        "profile"
    );
    assert_eq!(
        SecurityRuleSource::Switch("checksum_on_copy").as_str(),
        "switch"
    );
}

#[test]
fn a_rule_reports_its_own_title_and_explanation() {
    // arrange
    let rule = SecurityRule {
        id: SecurityRuleId::ChecksumOnCopy,
        state: SecurityRuleState::Enabled,
        source: SecurityRuleSource::Switch("checksum_on_copy"),
    };

    // act, assert
    assert!(rule.is_enabled());
    assert_eq!(rule.title(), "Checksum on copy");
    assert!(rule.explanation().contains("checksums"));
}
