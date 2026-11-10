use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::layout::rule::PluginImplUnit;
use crate::test::dependency::util::read_role_context;
use crate::workspace::model::DependencyKind;
use crate::workspace::util::parse_toml_file;

fn enable_plugin_product(context: &mut RuleContext) {
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nproduct-kind='platform-plugin'",
    )
    .expect("manifest");
}

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

#[test]
fn given_core_role_in_a_plugin_product_when_checked_then_the_unit_is_reported() {
    let mut context = read_role_context(Role::Context, Role::Api, DependencyKind::Normal);
    enable_plugin_product(&mut context);
    context.workspace.members[0].directory = "modules/core".to_owned();
    assert!(PluginImplUnit.check(&context).is_empty());
    context.project_config.crates[0].role = Some(Role::Core);
    let findings = PluginImplUnit.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "PluginImplUnit");
    assert_eq!(findings[0].finding.subject, "case-consumer");
    context.project_config.crates[0].role = Some(Role::Impl);
    assert_eq!(PluginImplUnit.check(&context).len(), 1);
}

#[test]
fn given_technology_unit_beside_api_when_an_impl_unit_exists_then_only_the_impl_is_reported() {
    let mut context = read_role_context(Role::Technology, Role::Api, DependencyKind::Normal);
    enable_plugin_product(&mut context);
    let mut implementation = context.workspace.members[0].clone();
    implementation.name = "case-impl".to_owned();
    implementation.directory = "modules/shop/implementation".to_owned();
    implementation.manifest.path = "modules/shop/implementation/Cargo.toml".to_owned();
    context.workspace.members.push(implementation);
    let mut implementation_config = context.project_config.crates[0].clone();
    implementation_config.name = "case-impl".to_owned();
    implementation_config.role = Some(Role::Impl);
    context.project_config.crates.push(implementation_config);
    let findings = PluginImplUnit.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "PluginImplUnit");
    assert_eq!(findings[0].finding.subject, "case-impl");
}
