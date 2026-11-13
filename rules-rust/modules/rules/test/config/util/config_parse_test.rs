use crate::check::model::CheckError;
use crate::config::model::ProjectConfig;
use crate::config::util::parse_project_config;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::{Dependency, DependencyKind};
use crate::workspace::util::parse_toml_file;

fn read_configuration(
    workspace_keys: &str,
    package_keys: &str,
) -> Result<ProjectConfig, CheckError> {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        &format!("[workspace.metadata.klibs-rules]\nproduct-kind = \"app\"\n{workspace_keys}\n"),
    )
    .expect("workspace manifest");
    let member = &mut context.workspace.members[0];
    member.manifest = parse_toml_file(
        "modules/sample/Cargo.toml",
        &format!("[package.metadata.klibs-rules]\n{package_keys}\n"),
    )
    .expect("package manifest");
    parse_project_config(&context.workspace)
}

fn read_configuration_error(workspace_keys: &str, package_keys: &str) -> String {
    read_configuration(workspace_keys, package_keys)
        .expect_err("the configuration is rejected")
        .to_string()
}

#[test]
fn given_role_kinds_in_both_metadata_tables_when_configuration_is_read_then_the_key_is_known() {
    let result = read_configuration("role-kinds = [\"registry\"]", "role-kinds = [\"codec\"]");
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn given_an_unknown_key_beside_role_kinds_when_configuration_is_read_then_it_is_still_an_error() {
    let message = read_configuration_error("role-kinds = [\"registry\"]\nrole-kind = []", "");
    assert!(message.contains("unknown key `role-kind`"), "{message}");
    assert!(message.contains("role-kinds"), "{message}");
}

#[test]
fn given_role_kinds_that_are_not_strings_when_configuration_is_read_then_reports_the_key() {
    let workspace = read_configuration_error("role-kinds = [1]", "");
    assert!(
        workspace.contains("`role-kinds` must be an array of strings"),
        "{workspace}"
    );
    let package = read_configuration_error("", "role-kinds = \"codec\"");
    assert!(
        package.contains("`role-kinds` must be an array of strings"),
        "{package}"
    );
}

#[test]
fn given_a_role_kind_equal_to_a_fixed_kind_word_when_configuration_is_read_then_reports_the_word() {
    for word in [
        "api", "model", "usecase", "util", "di", "stub", "fake", "table",
    ] {
        let workspace = read_configuration_error(&format!("role-kinds = [\"{word}\"]"), "");
        assert!(workspace.contains(&format!("\"{word}\"")), "{workspace}");
        assert!(workspace.contains("fixed kind"), "{workspace}");
    }
    let package = read_configuration_error("", "role-kinds = [\"entity\"]");
    assert!(package.contains("fixed kind"), "{package}");
}

#[test]
fn given_a_role_kind_equal_to_an_entry_point_word_when_configuration_is_read_then_reports_the_word()
{
    let built_in = read_configuration_error("role-kinds = [\"command\"]", "");
    assert!(built_in.contains("entry-point word"), "{built_in}");
    let workspace = read_configuration_error(
        "entry-point-words = [\"slot\"]\nrole-kinds = [\"slot\"]",
        "",
    );
    assert!(workspace.contains("\"slot\""), "{workspace}");
    assert!(workspace.contains("entry-point word"), "{workspace}");
    let package = read_configuration_error(
        "",
        "entry-point-words = [\"menu\"]\nrole-kinds = [\"menu\"]",
    );
    assert!(package.contains("entry-point word"), "{package}");
}

#[test]
fn given_a_role_kind_equal_to_a_technology_word_when_configuration_is_read_then_reports_the_word() {
    let built_in = read_configuration_error("role-kinds = [\"file\"]", "");
    assert!(built_in.contains("technology word"), "{built_in}");
    let workspace = read_configuration_error(
        "technology-words = [\"mongo\"]\nrole-kinds = [\"mongo\"]",
        "",
    );
    assert!(workspace.contains("\"mongo\""), "{workspace}");
    assert!(workspace.contains("technology word"), "{workspace}");
    let package = read_configuration_error(
        "",
        "technology-words = [\"redis\"]\nrole-kinds = [\"redis\"]",
    );
    assert!(package.contains("technology word"), "{package}");
}

#[test]
fn given_a_role_kind_that_is_empty_or_not_an_identifier_when_configuration_is_read_then_reports_the_word()
 {
    for word in [
        "",
        "two words",
        "9lives",
        "rate-limiter",
        "type",
        "r#limiter",
        "limiter::kind",
    ] {
        let workspace =
            read_configuration_error(&format!("role-kinds = [\"limiter\", \"{word}\"]"), "");
        assert!(workspace.contains(&format!("\"{word}\"")), "{workspace}");
        assert!(
            workspace.contains("not a valid Rust identifier"),
            "{workspace}"
        );
    }
    let package = read_configuration_error("", "role-kinds = [\"\"]");
    assert!(package.contains("not a valid Rust identifier"), "{package}");
}

#[test]
fn given_a_role_kind_named_like_a_root_block_when_configuration_is_read_then_reports_the_word() {
    for word in ["test", "fixture"] {
        let workspace = read_configuration_error(&format!("role-kinds = [\"{word}\"]"), "");
        assert!(workspace.contains(&format!("\"{word}\"")), "{workspace}");
        assert!(workspace.contains("block name"), "{workspace}");
    }
    let package = read_configuration_error("", "role-kinds = [\"fixture\"]");
    assert!(package.contains("block name"), "{package}");
}

#[test]
fn given_a_role_kind_in_snake_case_or_with_digits_when_configuration_is_read_then_it_is_accepted() {
    let result = read_configuration(
        "role-kinds = [\"rate_limiter\", \"codec2\", \"_hidden\"]",
        "",
    );
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn given_dependencies_when_configuration_is_read_then_only_external_non_development_technology_words_are_derived()
 {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    let name = context.workspace.members[0].name.clone();
    let dependencies = &mut context.workspace.members[0].dependencies;
    for dependency in ["aws-sdk-s3", "async-trait", "network", "local", &name] {
        dependencies.push(Dependency {
            name: dependency.to_owned(),
            code_name: dependency.replace('-', "_"),
            path: if dependency == name {
                Some("modules/sample".to_owned())
            } else {
                None
            },
            kind: DependencyKind::Normal,
            features: Vec::new(),
            is_target_specific: false,
            line: None,
        });
    }
    dependencies.push(Dependency {
        name: "test-double".to_owned(),
        code_name: "test_double".to_owned(),
        path: None,
        kind: DependencyKind::Development,
        features: Vec::new(),
        is_target_specific: false,
        line: None,
    });
    let config = parse_project_config(&context.workspace).expect("valid configuration");
    let words = &config.crates[0].technology_words;
    for included in [
        "aws",
        "aws_sdk_s3",
        "async_trait",
        "file",
        "memory",
        "system",
    ] {
        assert!(words.iter().any(|word| word == included));
    }
    for excluded in [
        "async",
        "test",
        "test_double",
        "network",
        "local",
        "case_sample",
    ] {
        assert!(!words.iter().any(|word| word == excluded));
    }
}
