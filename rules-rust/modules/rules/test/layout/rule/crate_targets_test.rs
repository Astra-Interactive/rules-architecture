use crate::check::api::Rule;
use crate::layout::rule::CrateTargets;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_multiple_and_custom_targets_when_checked_then_each_invalid_target_is_reported() {
    let context = read_case_context("CrateTargets", "fail-target-shapes");
    let findings = CrateTargets.check(&context);
    assert_eq!(findings.len(), 4);
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.message.contains("src/bin/ is not used"))
    );
    let mut context = read_case_context("CrateTargets", "pass-single-targets");
    assert!(CrateTargets.check(&context).is_empty());
    context.workspace.members[0].directory = String::new();
    context.workspace.members[0].targets[0].src_path = "src/lib.rs".to_owned();
    assert!(CrateTargets.check(&context).is_empty());
}

fn build_member_target(
    directory: &str,
    kind: &str,
    name: &str,
    path: &str,
) -> crate::workspace::model::Target {
    crate::workspace::model::Target {
        name: name.to_owned(),
        kinds: vec![kind.to_owned()],
        src_path: format!("{directory}/{path}"),
    }
}

#[test]
fn given_test_target_outside_the_tests_directory_when_checked_then_only_that_target_is_reported() {
    let mut context = read_case_context("CrateTargets", "pass-single-targets");
    let directory = context.workspace.members[0].directory.clone();
    let name = context.workspace.members[0].name.clone();
    context.workspace.members[0].targets.extend([
        build_member_target(&directory, "bench", "speed", "benches/speed.rs"),
        build_member_target(&directory, "example", "demo", "examples/demo.rs"),
        build_member_target(&directory, "test", "integration", "tests/integration.rs"),
        build_member_target(&directory, "test", "e2e", "e2e/api.rs"),
    ]);
    let findings = CrateTargets.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.rule_id.0, "CrateTargets");
    assert_eq!(findings[0].finding.subject, format!("{name} e2e"));
}
