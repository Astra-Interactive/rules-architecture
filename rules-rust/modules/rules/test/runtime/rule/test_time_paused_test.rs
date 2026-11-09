use crate::runtime::rule::TestTimePaused;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &TestTimePaused,
        "TestTimePaused",
        "#[tokio::test]\nasync fn test() {\n    tokio::time::sleep(std::time::Duration::ZERO).await;\n}\n",
        &[3],
    );
    assert_code_findings(
        &TestTimePaused,
        "TestTimePaused",
        "fn sleep() {}\n#[test]\nfn test() { sleep(); }\n",
        &[],
    );
}

#[test]
fn given_paused_runtime_configuration_when_checked_then_checks_flavor_and_development_feature() {
    use crate::check::api::Rule;
    use crate::test::declaration::util::read_code_context;
    use crate::workspace::model::{Dependency, DependencyKind};
    let source = "#[tokio::test(start_paused = true)]\nasync fn test() { tokio::time::sleep(std::time::Duration::ZERO).await; }";
    let mut context = read_code_context(source);
    assert_eq!(1, TestTimePaused.check(&context).len());
    context
        .workspace
        .members
        .iter_mut()
        .find(|member| member.name == context.indexes[0].crate_name)
        .expect("owner")
        .dependencies
        .push(Dependency {
            name: "tokio".to_owned(),
            code_name: "tokio".to_owned(),
            path: None,
            kind: DependencyKind::Development,
            features: vec!["test-util".to_owned()],
            is_target_specific: false,
            line: None,
        });
    assert!(TestTimePaused.check(&context).is_empty());
    assert_code_findings(
        &TestTimePaused,
        "TestTimePaused",
        "#[tokio::test(start_paused = true, flavor = \"multi_thread\")]\nasync fn test() {}",
        &[1],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&TestTimePaused, "TestTimePaused", "bad");
    assert_code_fixture(&TestTimePaused, "TestTimePaused", "good");
}
