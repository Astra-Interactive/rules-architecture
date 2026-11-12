use crate::runtime::rule::WatchTimeout;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_futures_crate_ready_futures_under_a_timeout_when_checked_then_each_is_reported() {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "async fn run(receiver: tokio::sync::watch::Receiver<bool>) {\n    tokio::time::timeout(std::time::Duration::ZERO, futures::future::ready(1));\n    tokio::time::timeout(std::time::Duration::ZERO, futures::future::ok::<u32, ()>(1));\n    tokio::time::timeout(std::time::Duration::ZERO, futures::future::err::<u32, ()>(()));\n    tokio::time::timeout(std::time::Duration::ZERO, futures::future::pending::<u32>());\n    tokio::time::timeout(std::time::Duration::ZERO, async { futures::future::ready(1).await });\n}",
        &[2, 3, 4, 6],
    );
}

#[test]
fn given_workspace_async_fn_without_await_under_a_timeout_when_checked_then_the_timeout_is_reported()
 {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "struct Gate { value: u32, receiver: tokio::sync::watch::Receiver<bool> }\nimpl Gate {\n    async fn snapshot(&self) -> u32 { self.value }\n    async fn wait(&mut self) { let _ = self.receiver.changed().await; }\n    async fn run(&mut self) {\n        tokio::time::timeout(std::time::Duration::ZERO, self.snapshot());\n        tokio::time::timeout(std::time::Duration::ZERO, self.wait());\n    }\n}\nasync fn settle() -> u32 { 1 }\nasync fn run_free() { tokio::time::timeout(std::time::Duration::ZERO, settle()); }",
        &[6, 11],
    );
}

#[test]
fn given_async_fns_suspending_in_join_or_select_when_under_timeout_then_reports_only_idle_one() {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "async fn joined() { tokio::join!(async {}, async {}); }\nasync fn selected() { tokio::select! { _ = tokio::task::yield_now() => {} } }\nasync fn idle() -> u32 { 1 }\nasync fn run() {\n    let limit = std::time::Duration::from_secs(1);\n    let _ = tokio::time::timeout(limit, joined()).await;\n    let _ = tokio::time::timeout(limit, selected()).await;\n    let _ = tokio::time::timeout(limit, idle()).await;\n}",
        &[8],
    );
}

#[test]
fn given_try_join_or_select_biased_fns_when_under_timeout_then_reports_only_idle_one() {
    assert_code_findings(
        &WatchTimeout,
        "WatchTimeout",
        "async fn tried() -> Result<(), ()> { tokio::try_join!(async { Ok::<(), ()>(()) }, async { Ok::<(), ()>(()) })?; Ok(()) }\nasync fn biased() { futures::select_biased! { _ = tokio::task::yield_now() => {} } }\nasync fn idle() -> u32 { 1 }\nasync fn run() {\n    let limit = std::time::Duration::from_secs(1);\n    let _ = tokio::time::timeout(limit, tried()).await;\n    let _ = tokio::time::timeout(limit, biased()).await;\n    let _ = tokio::time::timeout(limit, idle()).await;\n}",
        &[8],
    );
}

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
