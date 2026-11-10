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

#[test]
fn given_broadcast_lag_named_recv_result_when_linted_then_reports_lagged_arm() {
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "use tokio::sync::broadcast::error::RecvError;\nasync fn run(mut receiver: tokio::sync::broadcast::Receiver<u32>) {\n    loop {\n        let result = receiver.recv().await;\n        match result {\n            Ok(_) => {}\n            Err(RecvError::Lagged(_)) => break,\n            Err(RecvError::Closed) => return,\n        }\n    }\n}",
        &[7],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "use tokio::sync::broadcast::error::RecvError;\nasync fn run(mut receiver: tokio::sync::broadcast::Receiver<u32>) {\n    loop {\n        let result = receiver.recv().await;\n        match result {\n            Ok(_) => {}\n            Err(RecvError::Lagged(_)) => continue,\n            Err(RecvError::Closed) => return,\n        }\n    }\n}",
        &[],
    );
}

#[test]
fn given_broadcast_lag_nested_match_in_lagged_arm_when_linted_then_reports_lagged_arm() {
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "use tokio::sync::broadcast::error::RecvError;\nasync fn run(mut receiver: tokio::sync::broadcast::Receiver<u32>, stop: bool) {\n    loop {\n        match receiver.recv().await {\n            Ok(_) => {}\n            Err(RecvError::Lagged(_)) => match stop { true => break, false => continue },\n            Err(RecvError::Closed) => return,\n        }\n    }\n}",
        &[6],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "use tokio::sync::broadcast::error::RecvError;\nasync fn run(mut receiver: tokio::sync::broadcast::Receiver<u32>, stop: bool) {\n    loop {\n        match receiver.recv().await {\n            Ok(_) => {}\n            Err(RecvError::Lagged(_)) => match stop { true => continue, false => {} },\n            Err(RecvError::Closed) => return,\n        }\n    }\n}",
        &[],
    );
}

#[test]
fn given_broadcast_lag_let_else_and_ok_forms_when_linted_then_reports_first_lag_stop() {
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "async fn consume(mut receiver: tokio::sync::broadcast::Receiver<u32>) {\n    loop {\n        let Ok(event) = receiver.recv().await else {\n            break;\n        };\n        let _ = event;\n    }\n}",
        &[3],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "async fn consume(mut receiver: tokio::sync::broadcast::Receiver<u32>) {\n    while let Some(event) = receiver.recv().await.ok() {\n        let _ = event;\n    }\n}",
        &[2],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "async fn consume(mut receiver: tokio::sync::mpsc::Receiver<u32>) {\n    loop {\n        let Some(event) = receiver.recv().await else {\n            break;\n        };\n        let _ = event;\n    }\n}",
        &[],
    );
}

#[test]
fn given_broadcast_lag_let_chain_condition_when_linted_then_reports_the_while() {
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "async fn pump(mut receiver: tokio::sync::broadcast::Receiver<u32>, active: &std::sync::atomic::AtomicBool) {\n    while let Ok(event) = receiver.recv().await && active.load(std::sync::atomic::Ordering::Relaxed) {\n        let _ = event;\n    }\n}",
        &[2],
    );
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "async fn pump(mut receiver: tokio::sync::mpsc::Receiver<u32>, active: &std::sync::atomic::AtomicBool) {\n    while let Ok(event) = receiver.recv().await && active.load(std::sync::atomic::Ordering::Relaxed) {\n        let _ = event;\n    }\n}",
        &[],
    );
}

#[test]
fn given_broadcast_lag_receiver_from_subscribe_when_linted_then_reports_the_while() {
    assert_code_findings(
        &BroadcastLag,
        "BroadcastLag",
        "struct Timer { events: tokio::sync::broadcast::Sender<u32> }\nimpl Timer {\n    async fn forward(&self) {\n        let mut receiver = self.events.subscribe();\n        while let Ok(event) = receiver.recv().await {\n            let _ = event;\n        }\n    }\n}",
        &[5],
    );
}
