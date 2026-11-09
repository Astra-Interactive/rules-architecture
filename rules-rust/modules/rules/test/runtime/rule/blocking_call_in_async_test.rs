use crate::check::api::Rule;
use crate::runtime::rule::BlockingCallInAsync;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};

#[test]
fn given_typed_standard_channels_when_checked_then_respects_async_and_nested_sync_contexts() {
    assert_code_findings(
        &BlockingCallInAsync,
        "BlockingCallInAsync",
        "async fn run(receiver: std::sync::mpsc::Receiver<u32>) {\n    let result = receiver.recv();\n    fn sync(receiver: std::sync::mpsc::Receiver<u32>) { let result = receiver.recv(); }\n    let deferred = || receiver.recv();\n}",
        &[2],
    );
    assert_code_findings(
        &BlockingCallInAsync,
        "BlockingCallInAsync",
        "trait Worker {\n    async fn run() { std::fs::read(\"file\"); }\n}",
        &[2],
    );
}

#[test]
fn given_async_ownership_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &BlockingCallInAsync,
        "BlockingCallInAsync",
        "use std::thread::sleep as pause;\nasync fn run() {\n    pause(std::time::Duration::ZERO);\n    fn sync() { pause(std::time::Duration::ZERO); }\n    let sync = || pause(std::time::Duration::ZERO);\n    let future = async { std::fs::read(\"file\"); };\n}\n",
        &[3, 6],
    );
    assert_code_findings(
        &BlockingCallInAsync,
        "BlockingCallInAsync",
        "mod local { pub fn sleep() {} }\nasync fn run() { local::sleep(); let future = tokio::time::sleep(std::time::Duration::ZERO); }\n",
        &[],
    );
}

#[test]
fn given_persistent_cases_when_checked_then_rejects_unexpected_unresolved_source() {
    let source = "use std::thread::sleep as pause;\nasync fn run() {\n    pause(std::time::Duration::ZERO);\n    fn sync() { pause(std::time::Duration::ZERO); }\n    let sync = || pause(std::time::Duration::ZERO);\n    let future = async { std::fs::read(\"file\"); };\n}\n";
    let context = read_code_context(source);
    for index in &context.indexes {
        assert!(index.unanalysable.is_empty(), "{:?}", index.unanalysable);
    }
    assert_eq!(2, BlockingCallInAsync.check(&context).len());
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&BlockingCallInAsync, "BlockingCallInAsync", "bad");
    assert_code_fixture(&BlockingCallInAsync, "BlockingCallInAsync", "good");
}
