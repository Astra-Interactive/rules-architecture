use crate::config::model::Role;
use crate::config::util::{detect_role, parse_project_config};
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_role_rows_when_detected_then_first_matching_path_target_and_dependency_row_wins() {
    let context = read_case_context("RootFileShape", "pass-tree");
    let mut member = context.workspace.members[0].clone();
    let platforms = vec!["bukkit".to_owned(), "sqlx".to_owned()];
    let technology = vec!["sqlx".to_owned(), "aws".to_owned(), "aws_sdk_s3".to_owned()];
    for (directory, expected) in [
        ("instances/runner", Role::EntryPoint),
        ("modules/core/ui", Role::CoreUi),
        ("modules/core/ui/theme", Role::CoreUi),
        ("modules/core/bukkit", Role::CorePlatform),
        ("modules/core/api", Role::CoreApi),
        ("modules/core", Role::CoreApi),
        ("modules/wire/contract", Role::Contract),
        ("modules/data/dao", Role::StorageContract),
        ("modules/billing/api", Role::Api),
        ("modules/billing/impl", Role::Impl),
        ("modules/billing/bukkit", Role::Platform),
        ("modules/data/sqlx", Role::Platform),
        ("modules/data/aws", Role::Technology),
        ("modules/command", Role::Context),
        ("modules/impl-ui", Role::Context),
    ] {
        member.directory = directory.to_owned();
        assert_eq!(
            detect_role(&member, &platforms, &technology),
            Some(expected),
            "{directory}"
        );
    }
    member.directory = "modules/data/sqlx".to_owned();
    assert_eq!(
        detect_role(&member, &[], &technology),
        Some(Role::Technology)
    );
    member.name = "xtask".to_owned();
    assert_eq!(
        detect_role(&member, &platforms, &technology),
        Some(Role::Tooling)
    );
    member.name = "case-macro".to_owned();
    member.targets[0].kinds = vec!["proc-macro".to_owned()];
    assert_eq!(
        detect_role(&member, &platforms, &technology),
        Some(Role::ProcMacro)
    );
    member.targets[0].kinds = vec!["cdylib".to_owned()];
    assert_eq!(
        detect_role(&member, &platforms, &technology),
        Some(Role::EntryPoint)
    );
}

#[test]
fn given_declared_roles_and_ui_options_when_parsed_then_declaration_wins_and_values_are_retained() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.members[0].name = "xtask".to_owned();
    context.workspace.members[0].directory = "instances/runner".to_owned();
    context.workspace.members[0].targets[0].kinds = vec!["proc-macro".to_owned()];
    for role in [
        "tooling",
        "proc-macro",
        "entry-point",
        "core-ui",
        "core-platform",
        "core-api",
        "core",
        "contract",
        "storage-contract",
        "api",
        "impl",
        "impl-ui",
        "platform",
        "technology",
        "context",
    ] {
        context.workspace.members[0].manifest = parse_toml_file(
            "modules/sample/Cargo.toml",
            &format!("[package.metadata.klibs-rules]\nrole='{role}'"),
        )
        .expect("manifest");
        let config = parse_project_config(&context.workspace).expect("valid role");
        assert_eq!(
            crate::config::util::format_role_name(config.crates[0].role.expect("role")),
            role
        );
    }
    context.workspace.members[0].manifest = parse_toml_file(
        "modules/sample/Cargo.toml",
        "[package.metadata.klibs-rules]\nrole='impl-ui'",
    )
    .expect("manifest");
    let config = parse_project_config(&context.workspace).expect("defaults");
    assert_eq!(config.crates[0].composition, "local");
    assert_eq!(config.crates[0].implementation_access, "siblings");
    context.workspace.members[0].manifest = parse_toml_file("modules/sample/Cargo.toml", "[package.metadata.klibs-rules]\nrole='impl-ui'\ncomposition='host'\nimplementation-access='none'").expect("manifest");
    let config = parse_project_config(&context.workspace).expect("explicit options");
    assert_eq!(config.crates[0].composition, "host");
    assert_eq!(config.crates[0].implementation_access, "none");
    for invalid in [
        "role='context'\ncomposition='host'",
        "role='impl-ui'\ncomposition='other'",
        "role='impl-ui'\nimplementation-access='all'",
        "role='api'\nimplementation-access='siblings'",
    ] {
        context.workspace.members[0].manifest = parse_toml_file(
            "modules/sample/Cargo.toml",
            &format!("[package.metadata.klibs-rules]\n{invalid}"),
        )
        .expect("manifest");
        assert!(
            parse_project_config(&context.workspace).is_err(),
            "{invalid}"
        );
    }
}
