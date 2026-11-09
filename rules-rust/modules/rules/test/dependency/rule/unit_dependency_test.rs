use crate::check::api::Rule;
use crate::config::model::{AllowedDependency, ManifestException, ManifestScope};
use crate::dependency::rule::UnitDependency;
use crate::suppression::model::{RuleActivity, ValidatedExpectations};
use crate::suppression::util::apply_suppressions;
use crate::test::check::usecase::read_case_context;
use std::collections::BTreeSet;

#[test]
fn given_restricted_edges_when_checked_then_renamed_inherited_target_and_role_edges_are_reported() {
    let context = read_case_context("UnitDependency", "fail-restricted-edges");
    let findings = UnitDependency.check(&context);
    let mut actual: Vec<String> = findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("dependency line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "modules/consumer/Cargo.toml:10 case-consumer -> xtask",
            "modules/consumer/Cargo.toml:13 case-consumer -> case-owner",
            "modules/consumer/Cargo.toml:9 case-consumer -> case-owner",
            "modules/macros/Cargo.toml:12 case-macros -> case-owner",
            "xtask/Cargo.toml:9 xtask -> case-owner",
        ]
    );
    let context = read_case_context("UnitDependency", "pass-development-fakes");
    assert!(UnitDependency.check(&context).is_empty());
}

#[test]
fn given_exact_manifest_edge_exceptions_when_applied_then_only_named_owner_provider_pairs_are_accepted()
 {
    let mut context = read_case_context("UnitDependency", "fail-restricted-edges");
    context
        .project_config
        .allowed_dependencies
        .push(AllowedDependency {
            from_crate: "case-consumer".to_owned(),
            to_crate: "case-owner".to_owned(),
            reason: "suspected false positive, reported as #14".to_owned(),
            path: "modules/consumer/Cargo.toml".to_owned(),
            line: 20,
        });
    context
        .project_config
        .manifest_exceptions
        .push(ManifestException {
            rule_id: "UnitDependency".to_owned(),
            scope: ManifestScope::Package("case-macros".to_owned()),
            subject: "case-macros -> case-owner".to_owned(),
            reason: "suspected false positive, reported as #15".to_owned(),
            path: "modules/macros/Cargo.toml".to_owned(),
            line: 20,
        });
    let outcome = apply_suppressions(
        UnitDependency.check(&context),
        ValidatedExpectations {
            slots: Vec::new(),
            findings: Vec::new(),
        },
        &context.project_config,
        &RuleActivity {
            active_rules: BTreeSet::from(["UnitDependency".to_owned()]),
            is_platform_words_empty: true,
        },
    );
    assert_eq!(outcome.accepted.len(), 3);
    assert!(
        outcome
            .accepted
            .iter()
            .all(|accepted| accepted.location.line == Some(20))
    );
    let mut subjects: Vec<&str> = outcome
        .violations
        .iter()
        .map(|finding| finding.subject.as_str())
        .collect();
    subjects.sort();
    assert_eq!(subjects, ["case-consumer -> xtask", "xtask -> case-owner"]);
}
