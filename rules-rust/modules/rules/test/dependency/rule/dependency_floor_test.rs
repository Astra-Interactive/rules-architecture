use crate::check::api::Rule;
use crate::dependency::rule::DependencyFloor;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("DependencyFloor", "fail-policy");
    assert!(!DependencyFloor.check(&fail).is_empty());
    let pass = read_case_context("DependencyFloor", "pass-policy");
    assert!(DependencyFloor.check(&pass).is_empty());
}

#[test]
fn given_locked_async_trait_versions_when_checked_then_every_below_floor_identity_is_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.lock_file = None;
    assert!(DependencyFloor.check(&context).is_empty());
    context.workspace.lock_file = Some(parse_toml_file("Cargo.lock", "[[package]]\nname='async-trait'\nversion='0.1.91'\n[[package]]\nname='async-trait'\nversion='0.1.92-alpha.1'\n[[package]]\nname='async-trait'\nversion='0.1.92'\n[[package]]\nname='async-trait'\nversion='0.2.0'\n[[package]]\nname='other'\nversion='0.0.1'").expect("lock"));
    let findings = DependencyFloor.check(&context);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].finding.subject, "async-trait 0.1.91");
    assert_eq!(findings[0].finding.location.path, "Cargo.lock");
    assert_eq!(findings[0].finding.location.line, Some(3));
    assert_eq!(findings[1].finding.subject, "async-trait 0.1.92-alpha.1");
    assert_eq!(findings[1].finding.location.line, Some(6));
    context.workspace.lock_file = Some(
        parse_toml_file(
            "Cargo.lock",
            "[[package]]\nname='async-trait'\nversion='0.1.92+build'",
        )
        .expect("lock"),
    );
    assert!(DependencyFloor.check(&context).is_empty());
}
