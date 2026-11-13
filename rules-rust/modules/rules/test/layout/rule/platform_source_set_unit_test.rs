use crate::check::api::Rule;
use crate::config::model::Role;
use crate::layout::rule::PlatformSourceSetUnit;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_policy_fixtures_when_checked_then_incorrect_is_detected_and_correct_is_accepted() {
    let fail = read_case_context("PlatformSourceSetUnit", "fail-policy");
    assert!(!PlatformSourceSetUnit.check(&fail).is_empty());
    let pass = read_case_context("PlatformSourceSetUnit", "pass-policy");
    assert!(PlatformSourceSetUnit.check(&pass).is_empty());
}

#[test]
fn given_target_named_member_when_platform_evidence_exists_then_entry_points_and_architectures_are_exempt()
 {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.workspace.members[0].directory = "modules/sample/linux".to_owned();
    assert!(PlatformSourceSetUnit.check(&context).is_empty());
    context.workspace.members[0].manifest = parse_toml_file(
        "modules/sample/linux/Cargo.toml",
        "[target.'cfg(unix)'.dependencies]\nexternal='1'",
    )
    .expect("manifest");
    let findings = PlatformSourceSetUnit.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case-sample");
    assert_eq!(
        findings[0].finding.location.path,
        "modules/sample/linux/Cargo.toml"
    );
    context.workspace.members[0].directory = "modules/simd/aarch64".to_owned();
    assert!(PlatformSourceSetUnit.check(&context).is_empty());
    context.workspace.members[0].directory = "instances/linux".to_owned();
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    assert!(PlatformSourceSetUnit.check(&context).is_empty());
}
