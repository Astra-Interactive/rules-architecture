use crate::check::api::Rule;
use crate::config::model::Role;
use crate::dependency::rule::TechnologyDependency;
use crate::test::dependency::util::read_role_context;
use crate::workspace::model::DependencyKind;

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context =
        crate::test::check::usecase::read_case_context("TechnologyDependency", "fail-phase3-role");
    let mut actual: Vec<String> = TechnologyDependency
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
    assert_eq!(actual, ["modules/sample/Cargo.toml:12 case-sample -> sqlx"]);
    let context =
        crate::test::check::usecase::read_case_context("TechnologyDependency", "pass-phase3-role");
    assert!(TechnologyDependency.check(&context).is_empty());
}

#[test]
fn given_owned_external_technology_when_checked_then_all_dependency_kinds_are_confined_to_owner_and_entry()
 {
    let mut context = read_role_context(Role::Context, Role::Technology, DependencyKind::Normal);
    context.workspace.members[1].directory = "modules/data/sqlx".to_owned();
    context.workspace.members[0].dependencies[0].name = "sqlx-postgres".to_owned();
    context.workspace.members[0].dependencies[0].code_name = "database_alias".to_owned();
    context.workspace.members[0].dependencies[0].path = None;
    for kind in [
        DependencyKind::Normal,
        DependencyKind::Build,
        DependencyKind::Development,
    ] {
        context.workspace.members[0].dependencies[0].kind = kind;
        let findings = TechnologyDependency.check(&context);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].finding.subject,
            "case-consumer -> sqlx-postgres"
        );
        assert_eq!(findings[0].finding.location.line, Some(12));
    }
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    assert!(TechnologyDependency.check(&context).is_empty());
    context.project_config.crates[0].role = Some(Role::Technology);
    context.workspace.members[0].directory = "modules/other/sqlx".to_owned();
    assert!(TechnologyDependency.check(&context).is_empty());
}
