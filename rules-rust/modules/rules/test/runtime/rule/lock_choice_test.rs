use crate::runtime::rule::LockChoice;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Shared { flag: tokio::sync::Mutex<bool> }\n",
        &[1],
    );
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Mutex<T>(T);\nstruct Shared { flag: Mutex<bool> }\n",
        &[],
    );
}

#[test]
fn given_tokio_guard_scopes_when_checked_then_requires_an_await_crossing_the_guard() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "async fn run(gate: tokio::sync::Mutex<Vec<u32>>) {\n    let guard = gate.lock().await;\n    let count = guard.len();\n}\n",
        &[2],
    );
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "async fn run(gate: tokio::sync::Mutex<Vec<u32>>) {\n    let guard = gate.lock().await;\n    tokio::time::sleep(std::time::Duration::ZERO).await;\n    let count = guard.len();\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&LockChoice, "LockChoice", "bad");
    assert_code_fixture(&LockChoice, "LockChoice", "good");
}
