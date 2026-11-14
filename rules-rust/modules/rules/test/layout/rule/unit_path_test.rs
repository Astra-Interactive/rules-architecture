use crate::check::api::Rule;
use crate::layout::rule::UnitPath;
use crate::test::check::usecase::read_case_context;

const PLACEMENT_MESSAGE: &str = "place entry points at instances/<name> and product crates at modules/<a>[/<b>[/<c>]]; no segment is shared";

#[test]
fn given_roles_and_paths_when_checked_then_only_the_two_documented_places_pass() {
    let context = read_case_context("UnitPath", "fail-placement");
    let findings = UnitPath.check(&context);
    let mut subjects: Vec<String> = findings
        .iter()
        .map(|finding| finding.finding.subject.clone())
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
            "wrong-entry"
        ]
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.message == PLACEMENT_MESSAGE),
        "{findings:?}"
    );
    let context = read_case_context("UnitPath", "pass-placement");
    assert!(UnitPath.check(&context).is_empty());
}

#[test]
fn given_crate_named_xtask_at_the_root_when_checked_then_it_is_misplaced_like_any_other() {
    let mut context = read_case_context("UnitPath", "pass-placement");
    let mut member = context.workspace.members[0].clone();
    member.name = "xtask".to_owned();
    member.directory = "xtask".to_owned();
    context.workspace.members.push(member);
    context.project_config =
        crate::config::util::parse_project_config(&context.workspace).expect("valid configuration");
    let findings = UnitPath.check(&context);
    assert_eq!(1, findings.len(), "{findings:?}");
    assert_eq!("xtask", findings[0].finding.subject);
    assert_eq!(PLACEMENT_MESSAGE, findings[0].finding.message);
}

#[test]
fn given_xtask_directory_without_a_member_when_checked_then_nothing_is_registered_for_it() {
    let mut context = read_case_context("UnitPath", "pass-placement");
    context.workspace.directories.push("xtask".to_owned());
    assert!(UnitPath.check(&context).is_empty());
}
