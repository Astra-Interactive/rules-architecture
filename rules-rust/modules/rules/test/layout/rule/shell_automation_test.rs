use crate::check::api::Rule;
use crate::layout::rule::ShellAutomation;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("ShellAutomation", "fail-policy");
    assert!(!ShellAutomation.check(&fail).is_empty());
    let pass = read_case_context("ShellAutomation", "pass-policy");
    assert!(ShellAutomation.check(&pass).is_empty());
}

#[test]
fn given_automation_paths_when_checked_then_only_direct_root_and_member_scripts_are_reported() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.files = [
        "build.sh",
        "Makefile",
        "makefile",
        "Justfile",
        "justfile",
        "modules/sample/run.sh",
        "scripts/build.sh",
        "modules/sample/scripts/run.sh",
        "modules/Makefile",
        "build.SH",
    ]
    .map(str::to_owned)
    .to_vec();
    let findings = ShellAutomation.check(&context);
    let subjects: Vec<&str> = findings
        .iter()
        .map(|finding| finding.finding.subject.as_str())
        .collect();
    assert_eq!(
        subjects,
        [
            "build.sh",
            "Makefile",
            "makefile",
            "Justfile",
            "justfile",
            "modules/sample/run.sh"
        ]
    );
    for finding in findings {
        assert_eq!(finding.finding.location.path, finding.finding.subject);
        assert_eq!(finding.finding.location.line, None);
    }
    context.workspace.files = [
        "scripts/build.sh",
        "modules/sample/scripts/run.sh",
        "modules/Makefile",
        "build.SH",
    ]
    .map(str::to_owned)
    .to_vec();
    assert!(ShellAutomation.check(&context).is_empty());
}
