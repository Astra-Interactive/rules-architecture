use crate::runtime::rule::WatchTimeout;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_ready_future_or_tautological_predicate_when_checked_then_reports_proven_immediacy() {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "async fn run(receiver: tokio::sync::watch::Receiver<bool>) {\n tokio::time::timeout(std::time::Duration::ZERO, std::future::ready(1));\n tokio::time::timeout(std::time::Duration::ZERO, receiver.wait_for(|ready| *ready || true));\n}",
        &[2, 3],
    );
}

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "async fn run(receiver: tokio::sync::watch::Receiver<bool>) {\n    tokio::time::timeout(std::time::Duration::ZERO, receiver.wait_for(|_| true));\n    tokio::time::timeout(std::time::Duration::ZERO, async { 1 });\n}\n",
        &[2, 3],
    );
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "async fn run(receiver: tokio::sync::watch::Receiver<bool>) {\n    tokio::time::timeout(std::time::Duration::ZERO, receiver.wait_for(|ready| *ready));\n    tokio::time::timeout(std::time::Duration::ZERO, async { receiver.changed().await });\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&WatchTimeout, "WatchTimeout", "bad");
    assert_code_fixture(&WatchTimeout, "WatchTimeout", "good");
}

#[test]
fn given_watch_timeout_receiver_from_subscribe_when_linted_then_reports_the_timeout() {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "struct Gate { state: tokio::sync::watch::Sender<bool> }\nimpl Gate {\n    async fn wait(&self) {\n        let mut receiver = self.state.subscribe();\n        tokio::time::timeout(std::time::Duration::ZERO, receiver.wait_for(|_| true));\n        tokio::time::timeout(std::time::Duration::ZERO, receiver.wait_for(|ready| *ready));\n    }\n}",
        &[5],
    );
}

#[test]
fn given_timeout_at_or_async_block_around_tautological_wait_when_checked_then_reports_both() {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "async fn read(mut receiver: tokio::sync::watch::Receiver<bool>, deadline: tokio::time::Instant) {\n    let first = tokio::time::timeout(std::time::Duration::from_secs(1), async { receiver.wait_for(|_| true).await }).await;\n    let second = tokio::time::timeout_at(deadline, receiver.wait_for(|_| true)).await;\n    let third = tokio::time::timeout(std::time::Duration::from_secs(1), async { receiver.wait_for(|ready| *ready).await }).await;\n    let fourth = tokio::time::timeout(std::time::Duration::from_secs(1), async { let marker = 1; receiver.wait_for(|_| true).await }).await;\n}",
        &[2, 3],
    );
}
