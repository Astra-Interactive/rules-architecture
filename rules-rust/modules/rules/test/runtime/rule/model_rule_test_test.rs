use crate::runtime::rule::ModelRuleTest;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "mod model { pub struct Money; } mod util { pub fn is_valid(_: &crate::model::Money) -> bool { true } } #[cfg(test)] mod test { mod util {\n    #[tokio::test]\n    async fn test() { let money = crate::model::Money; let valid = crate::util::is_valid(&money); assert!(valid); }\n} }\n",
        &[2],
    );
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "#[cfg(test)] mod test { mod util {\n    #[test]\n    fn test() {}\n} }\n",
        &[],
    );
}

#[test]
fn given_async_io_test_in_util_when_checked_then_does_not_classify_it_as_model_only() {
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "#[cfg(test)] mod test { mod util {\n #[tokio::test] async fn verify_io() { let bytes = tokio::fs::read(\"file\").await; assert!(bytes.is_ok()); }\n} }",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ModelRuleTest, "ModelRuleTest", "bad");
    assert_code_fixture(&ModelRuleTest, "ModelRuleTest", "good");
}
