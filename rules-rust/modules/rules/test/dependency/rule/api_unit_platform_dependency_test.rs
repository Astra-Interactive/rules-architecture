use crate::check::api::Rule;
use crate::config::model::Role;
use crate::dependency::rule::ApiUnitPlatformDependency;
use crate::test::dependency::util::read_role_context;
use crate::workspace::model::DependencyKind;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_exact_platform_edge_exception_when_words_empty_then_expectation_stays_unfulfilled_with_inactive_reason()
 {
    let mut context = read_role_context(Role::Api, Role::Context, DependencyKind::Normal);
    context.workspace.members[0].dependencies[0].name = "bukkit-api".to_owned();
    context.workspace.members[0].dependencies[0].path = None;
    context
        .project_config
        .manifest_exceptions
        .push(crate::config::model::ManifestException {
            rule_id: "ApiUnitPlatformDependency".to_owned(),
            scope: crate::config::model::ManifestScope::Package("case-consumer".to_owned()),
            subject: "case-consumer -> bukkit-api".to_owned(),
            reason: "suspected false positive, reported as #43".to_owned(),
            path: "modules/billing/consumer/Cargo.toml".to_owned(),
            line: 20,
        });
    for words in [Vec::new(), vec!["bukkit".to_owned()]] {
        context.project_config.platform_words = words;
        let empty = context.project_config.platform_words.is_empty();
        let outcome = crate::suppression::util::apply_suppressions(
            ApiUnitPlatformDependency.check(&context),
            crate::suppression::model::ValidatedExpectations {
                slots: Vec::new(),
                findings: Vec::new(),
            },
            &context.project_config,
            &crate::suppression::model::RuleActivity {
                active_rules: std::collections::BTreeSet::from([
                    "ApiUnitPlatformDependency".to_owned()
                ]),
                is_platform_words_empty: empty,
            },
        );
        if empty {
            assert_eq!(outcome.violations.len(), 1);
            assert_eq!(outcome.violations[0].rule_id.0, "UnfulfilledExpectation");
            assert!(outcome.violations[0].message.contains("platform-words"));
        } else {
            assert!(outcome.violations.is_empty());
            assert_eq!(outcome.accepted.len(), 1);
            assert_eq!(outcome.accepted[0].subject, "case-consumer -> bukkit-api");
            assert_eq!(outcome.accepted[0].location.line, Some(20));
        }
    }
}

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context = crate::test::check::usecase::read_case_context(
        "ApiUnitPlatformDependency",
        "fail-phase3-role",
    );
    let mut actual: Vec<String> = ApiUnitPlatformDependency
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.unwrap_or(0),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        ["modules/sample/Cargo.toml:12 case-sample -> bukkit-api"]
    );
    let context = crate::test::check::usecase::read_case_context(
        "ApiUnitPlatformDependency",
        "pass-phase3-role",
    );
    assert!(ApiUnitPlatformDependency.check(&context).is_empty());
}

#[test]
fn given_platform_dependency_kinds_when_checked_then_app_target_normal_is_the_only_api_exception() {
    let mut context = read_role_context(Role::Api, Role::Context, DependencyKind::Normal);
    context.workspace.members[0].dependencies[0].name = "bukkit-api".to_owned();
    context.workspace.members[0].dependencies[0].code_name = "host_alias".to_owned();
    context.workspace.members[0].dependencies[0].path = None;
    assert!(ApiUnitPlatformDependency.check(&context).is_empty());
    context.project_config.platform_words = vec!["bukkit".to_owned()];
    for kind in [
        DependencyKind::Normal,
        DependencyKind::Build,
        DependencyKind::Development,
    ] {
        for target in [false, true] {
            context.workspace.members[0].dependencies[0].kind = kind;
            context.workspace.members[0].dependencies[0].is_target_specific = target;
            let findings = ApiUnitPlatformDependency.check(&context);
            assert_eq!(
                findings.is_empty(),
                kind == DependencyKind::Normal && target
            );
            if let Some(finding) = findings.first() {
                assert_eq!(finding.finding.subject, "case-consumer -> bukkit-api");
                assert_eq!(finding.finding.location.line, Some(12));
            }
        }
    }
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nproduct-kind='platform-plugin'",
    )
    .expect("manifest");
    context.workspace.members[0].dependencies[0].kind = DependencyKind::Normal;
    context.workspace.members[0].dependencies[0].is_target_specific = true;
    assert_eq!(ApiUnitPlatformDependency.check(&context).len(), 1);
    for role in [Role::CoreApi, Role::StorageContract, Role::Contract] {
        context.project_config.crates[0].role = Some(role);
        assert_eq!(ApiUnitPlatformDependency.check(&context).len(), 1);
    }
    context.project_config.platform_words.clear();
    assert!(ApiUnitPlatformDependency.check(&context).is_empty());
}
