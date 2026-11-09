use crate::check::api::Rule;
use crate::config::model::Role;
use crate::dependency::rule::EntryPointDependency;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::{Dependency, DependencyKind};

#[test]
fn given_entry_point_and_host_targets_when_depended_on_then_all_dependency_kinds_are_reported() {
    let mut context = read_case_context("EntryPointDependency", "fail-entry-and-host-dependencies");
    let mut subjects: Vec<String> = EntryPointDependency
        .check(&context)
        .into_iter()
        .map(|finding| finding.finding.subject)
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "consumer -> entry-lib",
            "consumer -> host",
            "consumer -> runner",
            "consumer -> static-host"
        ]
    );
    for config in &mut context.project_config.crates {
        config.role = Some(Role::Context);
    }
    assert_eq!(EntryPointDependency.check(&context).len(), 3);
    for member in &mut context.workspace.members {
        for dependency in &mut member.dependencies {
            dependency.path = None;
        }
    }
    assert!(EntryPointDependency.check(&context).is_empty());
}

#[test]
fn given_rust_libraries_and_external_edges_when_checked_then_no_entry_point_dependency_is_reported()
{
    let mut context = read_case_context("EntryPointDependency", "pass-library-dependencies");
    context.workspace.members[0].dependencies.push(Dependency {
        name: "external".to_owned(),
        path: None,
        kind: DependencyKind::Normal,
        is_target_specific: false,
        line: None,
    });
    assert!(EntryPointDependency.check(&context).is_empty());
}
