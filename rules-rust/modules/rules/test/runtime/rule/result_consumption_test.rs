use crate::runtime::rule::ResultConsumption;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_typed_runtime_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "#[expect(clippy::let_underscore_must_use, reason = \"too wide\")]\nfn run() {}\n",
        &[1],
    );
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn run() {\n    #[expect(clippy::let_underscore_must_use, reason = \"too broad\")]\n    let _ = std::fs::read(\"file\");\n}\n",
        &[2],
    );
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn run(sender: tokio::sync::oneshot::Sender<u32>) {\n    #[expect(clippy::let_underscore_must_use, reason = \"receiver is gone\")]\n    let _ = sender.send(1);\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ResultConsumption, "ResultConsumption", "bad");
    assert_code_fixture(&ResultConsumption, "ResultConsumption", "good");
}
