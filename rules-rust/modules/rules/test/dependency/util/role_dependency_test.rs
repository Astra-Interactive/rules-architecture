use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::dependency::rule::UnitDependency;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::{Dependency, DependencyKind};

const ROLE_VARIANTS: [Role; 15] = [
    Role::EntryPoint,
    Role::Platform,
    Role::Context,
    Role::Api,
    Role::Impl,
    Role::ImplUi,
    Role::Technology,
    Role::StorageContract,
    Role::Contract,
    Role::Core,
    Role::CoreApi,
    Role::CorePlatform,
    Role::CoreUi,
    Role::ProcMacro,
    Role::Tooling,
];
const ROLE_ROWS: &[(Role, &[Role])] = &[
    (
        Role::EntryPoint,
        &[
            Role::Platform,
            Role::Context,
            Role::Api,
            Role::Impl,
            Role::ImplUi,
            Role::Technology,
            Role::StorageContract,
            Role::Contract,
            Role::Core,
            Role::CoreApi,
            Role::CorePlatform,
            Role::CoreUi,
            Role::ProcMacro,
        ],
    ),
    (
        Role::Platform,
        &[
            Role::Api,
            Role::CoreApi,
            Role::Core,
            Role::CorePlatform,
            Role::StorageContract,
            Role::ProcMacro,
        ],
    ),
    (
        Role::Context,
        &[
            Role::Api,
            Role::CoreApi,
            Role::Core,
            Role::CorePlatform,
            Role::StorageContract,
            Role::ProcMacro,
        ],
    ),
    (
        Role::Api,
        &[
            Role::Api,
            Role::CoreApi,
            Role::Core,
            Role::StorageContract,
            Role::ProcMacro,
        ],
    ),
    (
        Role::Impl,
        &[Role::Api, Role::CoreApi, Role::Core, Role::ProcMacro],
    ),
    (
        Role::ImplUi,
        &[
            Role::Api,
            Role::CoreApi,
            Role::Core,
            Role::CoreUi,
            Role::Impl,
            Role::ProcMacro,
        ],
    ),
    (
        Role::Technology,
        &[
            Role::Api,
            Role::StorageContract,
            Role::CoreApi,
            Role::Core,
            Role::ProcMacro,
        ],
    ),
    (
        Role::StorageContract,
        &[Role::CoreApi, Role::Core, Role::ProcMacro],
    ),
    (Role::Contract, &[Role::ProcMacro]),
    (Role::Core, &[Role::ProcMacro]),
    (Role::CoreApi, &[Role::ProcMacro]),
    (Role::CorePlatform, &[Role::CoreApi, Role::ProcMacro]),
    (Role::CoreUi, &[Role::CoreApi, Role::ProcMacro]),
    (Role::ProcMacro, &[]),
    (Role::Tooling, &[]),
];

fn read_feature_context(table: &str, dependency: &str) -> RuleContext {
    use crate::workspace::api::WorkspaceReader;
    use crate::workspace::cargo::CargoWorkspaceReader;
    use crate::workspace::util::resolve_cargo_path;
    use std::fs;

    let root = tempfile::tempdir().expect("feature workspace");
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nresolver=\"3\"\nmembers=[\"consumer\",\"provider\"]\n",
    )
    .expect("workspace manifest");
    for name in ["consumer", "provider"] {
        fs::create_dir_all(root.path().join(name).join("src")).expect("member source");
        fs::write(root.path().join(name).join("src/lib.rs"), "").expect("valid source");
    }
    fs::write(root.path().join("provider/Cargo.toml"), "[package]\nname=\"case-provider\"\nedition=\"2024\"\n[features]\ndefault=[\"testing\"]\ntesting=[\"nested\"]\nnested=[\"fake\"]\nfake=[]\nsafe=[]\n").expect("provider manifest");
    fs::write(root.path().join("consumer/Cargo.toml"), format!("[package]\nname=\"case-consumer\"\nedition=\"2024\"\n[{table}]\nrenamed={{package=\"case-provider\",path=\"../provider\"{dependency}}}\n")).expect("consumer manifest");
    let mut context = read_role_context(Role::Api, Role::Api, DependencyKind::Normal);
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("Cargo-valid feature graph");
    context
}

#[test]
fn given_provider_feature_aliases_and_defaults_when_checked_then_fake_activation_is_production_only()
 {
    for (table, dependency, forbidden) in [
        (
            "dependencies",
            ",default-features=false,features=[\"testing\"]",
            true,
        ),
        ("dependencies", "", true),
        ("dependencies", ",default-features=false", false),
        (
            "dependencies",
            ",default-features=false,features=[\"safe\"]",
            false,
        ),
        ("dev-dependencies", ",features=[\"testing\"]", false),
        (
            "build-dependencies",
            ",default-features=false,features=[\"testing\"]",
            true,
        ),
    ] {
        let context = read_feature_context(table, dependency);
        let findings = UnitDependency.check(&context);
        assert_eq!(forbidden, !findings.is_empty(), "{table} {dependency}");
        if let Some(finding) = findings.first() {
            assert_eq!("UnitDependency", finding.finding.rule_id.0);
            assert_eq!("consumer/Cargo.toml", finding.finding.location.path);
            assert_eq!(Some(5), finding.finding.location.line);
            assert_eq!("case-consumer -> case-provider", finding.finding.subject);
        }
    }
}

pub(crate) fn read_role_context(from: Role, to: Role, kind: DependencyKind) -> RuleContext {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.indexes.clear();
    let mut provider = context.workspace.members[0].clone();
    provider.name = "case-provider".to_owned();
    provider.directory = "modules/billing/provider".to_owned();
    provider.manifest.path = "modules/billing/provider/Cargo.toml".to_owned();
    provider.dependencies.clear();
    context.workspace.members[0].name = "case-consumer".to_owned();
    context.workspace.members[0].directory = "modules/billing/consumer".to_owned();
    context.workspace.members[0].manifest.path = "modules/billing/consumer/Cargo.toml".to_owned();
    context.workspace.members[0].dependencies = vec![Dependency {
        name: provider.name.clone(),
        code_name: "case_provider".to_owned(),
        path: Some(provider.directory.clone()),
        kind,
        features: Vec::new(),
        is_target_specific: false,
        line: Some(12),
    }];
    context.workspace.members.push(provider);
    let mut provider_config = context.project_config.crates[0].clone();
    provider_config.name = "case-provider".to_owned();
    provider_config.role = Some(to);
    context.project_config.crates[0].name = "case-consumer".to_owned();
    context.project_config.crates[0].role = Some(from);
    context.project_config.crates.push(provider_config);
    context
}

#[test]
fn given_persistent_role_rows_when_checked_then_each_forbidden_edge_and_external_or_allowed_edge_matches()
 {
    for role in [
        "entry-point",
        "platform",
        "context",
        "api",
        "impl",
        "impl-ui",
        "technology",
        "storage-contract",
        "contract",
        "core",
        "core-api",
        "core-platform",
        "core-ui",
        "proc-macro",
        "tooling",
    ] {
        let fail = read_case_context("UnitDependency", &format!("fail-role-{role}"));
        let findings = UnitDependency.check(&fail);
        assert_eq!(findings.len(), 1, "{role}");
        assert_eq!(
            findings[0].finding.subject,
            "case-billing-consumer -> case-billing-provider"
        );
        assert_eq!(
            findings[0].finding.location.path,
            "modules/billing/consumer/Cargo.toml"
        );
        assert_eq!(findings[0].finding.location.line, Some(12));
        let pass = read_case_context("UnitDependency", &format!("pass-role-{role}"));
        assert!(UnitDependency.check(&pass).is_empty(), "{role}");
    }
}

#[test]
fn given_route_or_technology_modules_when_checked_then_contract_access_is_production_only() {
    let mut context = read_role_context(Role::Context, Role::Contract, DependencyKind::Normal);
    assert_eq!(UnitDependency.check(&context).len(), 1);
    for source in ["mod route {}", "mod file {}"] {
        let source = crate::test::module::util::read_shape_context(source, &[]);
        context.indexes = source.indexes;
        context.indexes[0].crate_name = "case-consumer".to_owned();
        assert!(UnitDependency.check(&context).is_empty());
    }
    let source = crate::test::module::util::read_shape_context(
        "#[cfg(test)] mod test { mod route {} }",
        &[],
    );
    context.indexes = source.indexes;
    context.indexes[0].crate_name = "case-consumer".to_owned();
    assert_eq!(UnitDependency.check(&context).len(), 1);
}

#[test]
fn given_allowed_dependency_when_suppressed_then_exact_edge_is_accepted_but_cycle_remains() {
    let mut context = read_role_context(Role::Core, Role::Context, DependencyKind::Normal);
    context.workspace.members[1].dependencies.push(Dependency {
        name: "case-consumer".to_owned(),
        code_name: "case_consumer".to_owned(),
        path: Some("modules/billing/consumer".to_owned()),
        kind: DependencyKind::Development,
        features: Vec::new(),
        is_target_specific: false,
        line: Some(14),
    });
    context
        .project_config
        .allowed_dependencies
        .push(crate::config::model::AllowedDependency {
            from_crate: "case-consumer".to_owned(),
            to_crate: "case-provider".to_owned(),
            reason: "suspected false positive, reported as #42".to_owned(),
            path: "modules/billing/consumer/Cargo.toml".to_owned(),
            line: 20,
        });
    let outcome = crate::suppression::util::apply_suppressions(
        UnitDependency.check(&context),
        crate::suppression::model::ValidatedExpectations {
            slots: Vec::new(),
            findings: Vec::new(),
        },
        &context.project_config,
        &crate::suppression::model::RuleActivity {
            active_rules: std::collections::BTreeSet::from(["UnitDependency".to_owned()]),
            is_platform_words_empty: true,
        },
    );
    assert!(outcome.violations.is_empty());
    assert_eq!(outcome.accepted.len(), 1);
    assert_eq!(outcome.accepted[0].location.line, Some(20));
    assert_eq!(
        crate::dependency::rule::UnitDependencyCycle
            .check(&context)
            .len(),
        1
    );
}

#[test]
fn given_every_production_role_pair_when_checked_then_exact_dependency_table_is_enforced() {
    for (from, allowed) in ROLE_ROWS {
        for to in ROLE_VARIANTS {
            for kind in [DependencyKind::Normal, DependencyKind::Build] {
                let context = read_role_context(*from, to, kind);
                let findings = UnitDependency.check(&context);
                assert_eq!(
                    findings.is_empty(),
                    allowed.contains(&to),
                    "{from:?} -> {to:?} {kind:?}"
                );
                if let Some(finding) = findings.first() {
                    assert_eq!(finding.finding.subject, "case-consumer -> case-provider");
                    assert_eq!(
                        finding.finding.location.path,
                        "modules/billing/consumer/Cargo.toml"
                    );
                    assert_eq!(finding.finding.location.line, Some(12));
                }
            }
        }
    }
}

#[test]
fn given_composition_sibling_and_development_edges_when_checked_then_only_documented_additions_pass()
 {
    let mut context = read_role_context(Role::Technology, Role::Api, DependencyKind::Normal);
    context.workspace.members[1].directory = "modules/other/api".to_owned();
    context.workspace.members[0].dependencies[0].path = Some("modules/other/api".to_owned());
    assert_eq!(UnitDependency.check(&context).len(), 1);
    let mut context = read_role_context(Role::ImplUi, Role::ImplUi, DependencyKind::Normal);
    assert_eq!(UnitDependency.check(&context).len(), 1);
    context.project_config.crates[0].composition = "host".to_owned();
    assert!(UnitDependency.check(&context).is_empty());
    let mut context = read_role_context(Role::ImplUi, Role::Impl, DependencyKind::Normal);
    assert!(UnitDependency.check(&context).is_empty());
    context.project_config.crates[0].implementation_access = "none".to_owned();
    assert_eq!(UnitDependency.check(&context).len(), 1);
    let mut context = read_role_context(
        Role::Contract,
        Role::Technology,
        DependencyKind::Development,
    );
    context.workspace.members[0].dependencies[0].features = vec!["fake".to_owned()];
    assert!(UnitDependency.check(&context).is_empty());
    context.workspace.members[0].dependencies[0].kind = DependencyKind::Build;
    assert_eq!(UnitDependency.check(&context).len(), 1);
}
