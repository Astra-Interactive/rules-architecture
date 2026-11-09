use crate::check::api::Rule;
use crate::layout::rule::PublishedVersion;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_workspace_publication_when_inherited_then_member_conflict_keeps_member_location() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.root_manifest =
        parse_toml_file("Cargo.toml", "[workspace.package]\npublish=false").expect("root manifest");
    context.workspace.members[0].manifest = parse_toml_file(
        "modules/sample/Cargo.toml",
        "[package]\nversion.workspace=true\npublish.workspace=true",
    )
    .expect("member manifest");
    let findings = PublishedVersion.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.location.path,
        "modules/sample/Cargo.toml"
    );
    assert_eq!(findings[0].finding.location.line, Some(2));
    context.workspace.members[0].manifest = parse_toml_file(
        "modules/sample/Cargo.toml",
        "[package]\npublish.workspace=true",
    )
    .expect("member manifest");
    assert!(PublishedVersion.check(&context).is_empty());
}

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("PublishedVersion", "fail-policy");
    assert!(!PublishedVersion.check(&fail).is_empty());
    let pass = read_case_context("PublishedVersion", "pass-policy");
    assert!(PublishedVersion.check(&pass).is_empty());
}

#[test]
fn given_publication_settings_when_checked_then_only_version_publication_conflicts_are_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    for settings in [
        "version='1.0.0'\npublish=false",
        "version.workspace=true\npublish=[]",
        "publish=true",
        "publish=['private']",
    ] {
        context.workspace.members[0].manifest = parse_toml_file(
            "modules/sample/Cargo.toml",
            &format!("[package]\n{settings}"),
        )
        .expect("manifest");
        let findings = PublishedVersion.check(&context);
        assert_eq!(findings.len(), 1, "{settings}");
        assert_eq!(findings[0].finding.subject, "case-sample");
        assert_eq!(
            findings[0].finding.location.path,
            "modules/sample/Cargo.toml"
        );
        assert_eq!(findings[0].finding.location.line, Some(2));
    }
    for settings in [
        "",
        "publish=false",
        "publish=[]",
        "version='1.0.0'",
        "version.workspace=true\npublish=true",
        "version='1.0.0'\npublish=['private']",
    ] {
        context.workspace.members[0].manifest = parse_toml_file(
            "modules/sample/Cargo.toml",
            &format!("[package]\n{settings}"),
        )
        .expect("manifest");
        assert!(PublishedVersion.check(&context).is_empty(), "{settings}");
    }
}
