use crate::check::api::Rule;
use crate::config::model::Role;
use crate::layout::rule::XtaskAlias;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("XtaskAlias", "fail-policy");
    assert!(!XtaskAlias.check(&fail).is_empty());
    let pass = read_case_context("XtaskAlias", "pass-policy");
    assert!(XtaskAlias.check(&pass).is_empty());
}

#[test]
fn given_xtask_member_when_effective_alias_is_checked_then_wrong_config_location_is_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    assert!(XtaskAlias.check(&context).is_empty());
    context.workspace.members[0].name = "xtask".to_owned();
    context.workspace.cargo_config = None;
    let findings = XtaskAlias.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, ".cargo/config.toml");
    assert_eq!(findings[0].finding.location.line, None);
    for path in [".cargo/config", ".cargo/config.toml"] {
        context.workspace.cargo_config =
            Some(parse_toml_file(path, "[alias]\nxtask='run --package wrong --'").expect("config"));
        let findings = XtaskAlias.check(&context);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding.subject, path);
        assert_eq!(findings[0].finding.location.path, path);
        assert_eq!(findings[0].finding.location.line, Some(2));
        context.workspace.cargo_config =
            Some(parse_toml_file(path, "[alias]\nxtask='run --package xtask --'").expect("config"));
        assert!(XtaskAlias.check(&context).is_empty());
    }
}

#[test]
fn given_array_form_of_the_alias_when_checked_then_only_a_wrong_array_is_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.members[0].name = "xtask".to_owned();
    context.workspace.cargo_config = Some(
        parse_toml_file(
            ".cargo/config.toml",
            "[alias]\nxtask = [\"run\", \"--package\", \"xtask\", \"--\"]",
        )
        .expect("config"),
    );
    assert!(XtaskAlias.check(&context).is_empty());
    context.workspace.cargo_config = Some(
        parse_toml_file(
            ".cargo/config.toml",
            "[alias]\nxtask = [\"run\", \"--package\", \"wrong\", \"--\"]",
        )
        .expect("config"),
    );
    let findings = XtaskAlias.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.location.line, Some(2));
}

#[test]
fn given_tooling_crate_with_a_prefixed_name_when_the_alias_is_missing_then_it_is_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.members[0].name = "shop-xtask".to_owned();
    context.project_config.crates[0].name = "shop-xtask".to_owned();
    context.project_config.crates[0].role = Some(Role::Tooling);
    context.workspace.cargo_config = None;
    let findings = XtaskAlias.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.rule_id.0, "XtaskAlias");
    assert_eq!(findings[0].finding.subject, ".cargo/config.toml");
}
