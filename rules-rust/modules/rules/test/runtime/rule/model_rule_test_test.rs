use crate::runtime::rule::ModelRuleTest;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "#[cfg(test)] mod test { mod util {\n    #[tokio::test]\n    async fn test() {}\n} }\n",
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
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ModelRuleTest, "ModelRuleTest", "bad");
    assert_code_fixture(&ModelRuleTest, "ModelRuleTest", "good");
}
