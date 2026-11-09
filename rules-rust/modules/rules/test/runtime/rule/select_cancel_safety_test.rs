use crate::runtime::rule::SelectCancelSafety;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "async fn run(gate: tokio::sync::Mutex<()>) {\n    loop { tokio::select! { _guard = gate.lock() => {}, else => {} } }\n}\n",
        &[2],
    );
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "async fn run(gate: tokio::sync::Mutex<()>) {\n    let future = gate.lock();\n    tokio::pin!(future);\n    loop { tokio::select! { _guard = &mut future => {}, else => {} } }\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&SelectCancelSafety, "SelectCancelSafety", "bad");
    assert_code_fixture(&SelectCancelSafety, "SelectCancelSafety", "good");
}
