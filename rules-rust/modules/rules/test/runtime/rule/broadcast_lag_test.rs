use crate::runtime::rule::BroadcastLag;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_broadcast_match_loop_when_checked_then_lag_continues_and_closed_may_stop() {
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "fn consume(_: u32) {} async fn run(mut receiver: tokio::sync::broadcast::Receiver<u32>) {\n loop { match receiver.recv().await { Ok(event) => consume(event), Err(_) => break } }\n}",
        &[2],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "fn consume(_: u32) {} use tokio::sync::broadcast::error::RecvError; async fn run(mut receiver: tokio::sync::broadcast::Receiver<u32>) {\n loop { match receiver.recv().await { Ok(event) => consume(event), Err(RecvError::Lagged(_)) => break, Err(RecvError::Closed) => break } }\n}",
        &[2],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "fn consume(_: u32) {} use tokio::sync::broadcast::error::RecvError; async fn run(mut receiver: tokio::sync::broadcast::Receiver<u32>) {\n loop { match receiver.recv().await { Ok(event) => consume(event), Err(RecvError::Lagged(_)) => continue, Err(RecvError::Closed) => break } }\n}",
        &[],
    );
}

#[test]
fn given_typed_runtime_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "async fn run(receiver: tokio::sync::broadcast::Receiver<u32>) {\n    while let Ok(event) = receiver.recv().await {}\n}\n",
        &[2],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "async fn run(receiver: tokio::sync::mpsc::Receiver<u32>) {\n    while let Ok(event) = receiver.recv().await {}\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&BroadcastLag, "BroadcastLag", "bad");
    assert_code_fixture(&BroadcastLag, "BroadcastLag", "good");
}
