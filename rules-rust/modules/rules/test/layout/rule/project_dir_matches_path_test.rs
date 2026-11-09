use crate::check::api::Rule;
use crate::layout::rule::ProjectDirMatchesPath;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_member_globs_when_checked_then_each_pattern_is_reported_at_its_manifest_line() {
    let context = read_case_context("ProjectDirMatchesPath", "fail-member-globs");
    let findings = ProjectDirMatchesPath.check(&context);
    assert_eq!(findings.len(), 3);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.location.line == Some(3))
    );
    let context = read_case_context("ProjectDirMatchesPath", "pass-explicit-members");
    assert!(ProjectDirMatchesPath.check(&context).is_empty());
}
