use crate::check::api::Rule;
use crate::layout::rule::NestedUnit;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_nested_and_sibling_crates_when_checked_then_only_inner_members_are_reported() {
    let mut nested = read_case_context("NestedUnit", "fail-inner-member");
    let findings = NestedUnit.check(&nested);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "inner");
    nested.workspace.members[0].directory = String::new();
    assert_eq!(NestedUnit.check(&nested).len(), 1);
    let siblings = read_case_context("NestedUnit", "pass-sibling-members");
    assert!(NestedUnit.check(&siblings).is_empty());
}
