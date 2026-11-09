use crate::check::api::Rule;
use crate::config::model::Role;
use crate::layout::rule::PluginImplUnit;
use crate::test::dependency::util::read_role_context;
use crate::workspace::model::DependencyKind;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context =
        crate::test::check::usecase::read_case_context("PluginImplUnit", "fail-phase3-role");
    let mut actual: Vec<String> = PluginImplUnit
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
    assert_eq!(actual, ["modules/sample/Cargo.toml:0 case-sample"]);
    let context =
        crate::test::check::usecase::read_case_context("PluginImplUnit", "pass-phase3-role");
    assert!(PluginImplUnit.check(&context).is_empty());
}

#[test]
fn given_plugin_units_when_checked_then_impls_and_nonplatform_api_siblings_are_reported() {
    let mut context = read_role_context(Role::Context, Role::Api, DependencyKind::Normal);
    assert!(PluginImplUnit.check(&context).is_empty());
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nproduct-kind='platform-plugin'",
    )
    .expect("manifest");
    let findings = PluginImplUnit.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case-consumer");
    assert_eq!(
        findings[0].finding.location.path,
        "modules/billing/consumer/Cargo.toml"
    );
    for role in [Role::Platform, Role::Api] {
        context.project_config.crates[0].role = Some(role);
        assert!(PluginImplUnit.check(&context).is_empty());
    }
    context.workspace.members[0].directory = "modules/other/implementation".to_owned();
    for role in [Role::Impl, Role::ImplUi] {
        context.project_config.crates[0].role = Some(role);
        assert_eq!(PluginImplUnit.check(&context).len(), 1);
    }
}
