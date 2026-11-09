use crate::check::api::Rule;
use crate::dependency::rule::UnitDependencyCycle;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::{Dependency, DependencyKind};

#[test]
fn given_development_cycles_when_checked_then_each_strong_component_is_reported_once() {
    let mut context = read_case_context("UnitDependencyCycle", "fail-development-cycles");
    let subjects: Vec<String> = UnitDependencyCycle
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    assert_eq!(subjects, ["alpha, beta, gamma", "delta, epsilon"]);
    for member in &mut context.workspace.members {
        for dependency in &mut member.dependencies {
            dependency.kind = DependencyKind::Normal;
        }
    }
    assert!(UnitDependencyCycle.check(&context).is_empty());
    context.workspace.members[0].dependencies.push(Dependency {
        name: "beta".to_owned(),
        code_name: "beta".to_owned(),
        path: None,
        kind: DependencyKind::Development,
        features: Vec::new(),
        is_target_specific: false,
        line: None,
    });
    assert!(UnitDependencyCycle.check(&context).is_empty());
}

#[test]
fn given_acyclic_and_external_edges_when_checked_then_no_cycle_is_reported() {
    let mut context = read_case_context("UnitDependencyCycle", "pass-acyclic-development");
    context.workspace.members[0].dependencies.push(Dependency {
        name: "external".to_owned(),
        code_name: "external".to_owned(),
        path: None,
        kind: DependencyKind::Development,
        features: Vec::new(),
        is_target_specific: true,
        line: None,
    });
    assert!(UnitDependencyCycle.check(&context).is_empty());
}
