use crate::check::api::Rule;
use crate::dependency::rule::TimeCrate;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::{Dependency, DependencyKind};

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("TimeCrate", "fail-policy");
    assert!(!TimeCrate.check(&fail).is_empty());
    let pass = read_case_context("TimeCrate", "pass-policy");
    assert!(TimeCrate.check(&pass).is_empty());
}

#[test]
fn given_time_dependencies_across_members_when_checked_then_only_multiple_non_dev_crates_are_reported()
 {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.members[0].dependencies.clear();
    for (name, kind) in [
        ("chrono", DependencyKind::Normal),
        ("jiff", DependencyKind::Development),
        ("chrono", DependencyKind::Build),
    ] {
        context.workspace.members[0].dependencies.push(Dependency {
            name: name.to_owned(),
            code_name: name.to_owned(),
            path: None,
            kind,
            features: Vec::new(),
            is_target_specific: true,
            line: Some(4),
        });
    }
    assert!(TimeCrate.check(&context).is_empty());
    let mut second = context.workspace.members[0].clone();
    second.name = "case-other".to_owned();
    second.dependencies = vec![Dependency {
        name: "time".to_owned(),
        code_name: "time".to_owned(),
        path: None,
        kind: DependencyKind::Build,
        features: Vec::new(),
        is_target_specific: true,
        line: Some(8),
    }];
    context.workspace.members.push(second);
    let findings = TimeCrate.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, ".");
    assert_eq!(findings[0].finding.location.path, "Cargo.toml");
    assert_eq!(findings[0].finding.location.line, None);
    assert!(findings[0].finding.message.contains("chrono, time"));
    context.workspace.members[1].dependencies[0].kind = DependencyKind::Development;
    assert!(TimeCrate.check(&context).is_empty());
}
