use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::layout::rule::RootPackagePath;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("RootPackagePath", "fail-policy");
    assert!(!RootPackagePath.check(&fail).is_empty());
    let pass = read_case_context("RootPackagePath", "pass-policy");
    assert!(RootPackagePath.check(&pass).is_empty());
}

#[test]
fn given_explicit_prefix_when_names_and_lib_override_are_checked_then_exact_member_is_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nprefix='shop'",
    )
    .expect("manifest");
    let findings = RootPackagePath.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case-sample");
    assert_eq!(
        findings[0].finding.location.path,
        "modules/sample/Cargo.toml"
    );
    context.workspace.members[0].name = "shop-sample".to_owned();
    assert!(RootPackagePath.check(&context).is_empty());
    context.workspace.members[0].manifest =
        parse_toml_file("modules/sample/Cargo.toml", "[lib]\nname='alternate'").expect("manifest");
    let findings = RootPackagePath.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.location.line, Some(2));
}

#[test]
fn given_root_package_without_container_when_checked_then_no_empty_suffix_name_is_invented() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nprefix='shop'",
    )
    .expect("prefix");
    context.workspace.members[0].directory = ".".to_owned();
    context.workspace.members[0].name = "shop".to_owned();
    assert!(RootPackagePath.check(&context).is_empty());
    context.workspace.members[0].manifest =
        parse_toml_file("Cargo.toml", "[lib]\nname='alternate'").expect("override");
    let findings = RootPackagePath.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.location.line, Some(2));
}

#[test]
fn given_derived_prefix_candidates_when_checked_then_majority_wins_and_ties_require_configuration()
{
    let mut context = read_case_context("RootFileShape", "pass-tree");
    assert!(RootPackagePath.check(&context).is_empty());
    let mut second = context.workspace.members[0].clone();
    second.directory = "modules/other".to_owned();
    second.name = "other-other".to_owned();
    context.workspace.members.push(second);
    let findings = RootPackagePath.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, ".");
    assert_eq!(findings[0].anchor, Anchor::Workspace);
    let mut third = context.workspace.members[0].clone();
    third.directory = "modules/third".to_owned();
    third.name = "case-third".to_owned();
    context.workspace.members.push(third);
    let findings = RootPackagePath.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "other-other");
    context.workspace.members.truncate(1);
    context.workspace.members[0].name = "no-candidate".to_owned();
    let findings = RootPackagePath.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, ".");
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nprefix='case'",
    )
    .expect("manifest");
    for reserved in ["std", "core", "alloc", "proc_macro", "proc-macro", "test"] {
        context.workspace.members[0].name = reserved.to_owned();
        let findings = RootPackagePath.check(&context);
        assert_eq!(findings.len(), 2);
        assert!(
            findings
                .iter()
                .any(|finding| finding.finding.subject == reserved)
        );
    }
}
