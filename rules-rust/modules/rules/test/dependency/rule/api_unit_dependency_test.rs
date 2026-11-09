use crate::check::api::Rule;
use crate::config::model::Role;
use crate::dependency::rule::ApiUnitDependency;
use crate::test::dependency::util::read_role_context;
use crate::workspace::model::DependencyKind;

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context =
        crate::test::check::usecase::read_case_context("ApiUnitDependency", "fail-phase3-role");
    let mut actual: Vec<String> = ApiUnitDependency
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
        ["modules/sample/Cargo.toml:12 case-sample -> case-provider"]
    );
    let context =
        crate::test::check::usecase::read_case_context("ApiUnitDependency", "pass-phase3-role");
    assert!(ApiUnitDependency.check(&context).is_empty());
}

#[test]
fn given_api_edges_when_checked_then_forbidden_role_reports_its_exact_edge_and_dev_is_free() {
    let mut context = read_role_context(Role::Api, Role::Impl, DependencyKind::Normal);
    let findings = ApiUnitDependency.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.subject,
        "case-consumer -> case-provider"
    );
    assert_eq!(findings[0].finding.location.line, Some(12));
    for role in [
        Role::Api,
        Role::CoreApi,
        Role::Core,
        Role::StorageContract,
        Role::ProcMacro,
    ] {
        context.project_config.crates[1].role = Some(role);
        assert!(ApiUnitDependency.check(&context).is_empty(), "{role:?}");
    }
    context.project_config.crates[1].role = Some(Role::Impl);
    context.workspace.members[0].dependencies[0].kind = DependencyKind::Development;
    assert!(ApiUnitDependency.check(&context).is_empty());
}
