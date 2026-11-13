use crate::check::api::Rule;
use crate::config::model::Role;
use crate::layout::rule::CrateTargets;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::Target;

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

#[test]
fn given_entry_point_crate_with_library_and_one_binary_when_checked_then_it_is_accepted() {
    let context = read_case_context("CrateTargets", "pass-binary-entry-point");
    let member = &context.workspace.members[0];
    assert_eq!(member.name, "shop-cli");
    assert!(member.targets.iter().any(Target::is_lib_target));
    assert_eq!(
        1,
        member
            .targets
            .iter()
            .filter(|target| target.is_bin())
            .count()
    );
    assert!(CrateTargets.check(&context).is_empty());
}

#[test]
fn given_library_and_binary_in_a_crate_of_another_role_when_checked_then_the_binary_reports_never_both()
 {
    for role in [Some(Role::Context), Some(Role::Api), None] {
        let mut context = read_case_context("CrateTargets", "pass-binary-entry-point");
        let name = context.workspace.members[0].name.clone();
        let config = context
            .project_config
            .crates
            .iter_mut()
            .find(|config| config.name == name)
            .expect("configuration of the entry-point crate");
        config.role = role;
        let findings = CrateTargets.check(&context);
        assert_eq!(findings.len(), 1, "{role:?}: {findings:?}");
        assert_eq!(findings[0].finding.subject, "shop-cli shop-cli");
        assert_eq!(
            findings[0].finding.message,
            "a crate is a library or a binary, never both"
        );
    }
}

#[test]
fn given_library_and_binary_under_modules_when_checked_then_never_both_is_still_reported() {
    let context = read_case_context("CrateTargets", "fail-shared-target");
    let findings = CrateTargets.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].finding.location.path,
        "modules/sample/src/main.rs"
    );
    assert_eq!(
        findings[0].finding.message,
        "a crate is a library or a binary, never both"
    );
}

#[test]
fn given_entry_point_crate_with_a_second_binary_when_checked_then_several_binaries_and_src_bin_report()
 {
    let mut context = read_case_context("CrateTargets", "pass-binary-entry-point");
    let directory = context.workspace.members[0].directory.clone();
    context.workspace.members[0]
        .targets
        .push(build_member_target(
            &directory,
            "bin",
            "extra",
            "src/bin/extra.rs",
        ));
    let findings = CrateTargets.check(&context);
    assert_eq!(findings.len(), 2, "{findings:?}");
    let by_target: Vec<(&str, &str)> = findings
        .iter()
        .map(|finding| {
            (
                finding.finding.subject.as_str(),
                finding.finding.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        by_target,
        [
            (
                "shop-cli shop-cli",
                "several binaries are separate entry-point crates"
            ),
            (
                "shop-cli extra",
                "a binary target starts at src/main.rs; several binaries are separate entry-point crates; src/bin/ is not used"
            ),
        ]
    );
}
