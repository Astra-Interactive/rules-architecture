use crate::check::api::Rule;
use crate::layout::rule::UnitPath;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_roles_and_paths_when_checked_then_wrong_placements_and_unregistered_tooling_are_reported()
{
    let context = read_case_context("UnitPath", "fail-placement");
    let mut subjects: Vec<String> = UnitPath
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "deep",
            "outside",
            "runner",
            "shared",
            "tools",
            "wrong-entry",
            "xtask"
        ]
    );
    let context = read_case_context("UnitPath", "pass-placement");
    assert!(UnitPath.check(&context).is_empty());
}
