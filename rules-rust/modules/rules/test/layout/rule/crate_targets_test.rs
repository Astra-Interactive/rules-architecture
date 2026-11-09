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
