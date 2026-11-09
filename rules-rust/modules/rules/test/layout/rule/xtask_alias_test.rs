use crate::check::api::Rule;
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
