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

#[test]
fn given_tdd_blocking_file_read_after_await_when_async_then_reports_read_call() {
    assert_code_findings(
        &BlockingCallInAsync,
        "BlockingCallInAsync",
        "use std::io::Read;\nasync fn read_file(mut file: std::fs::File) -> std::io::Result<Vec<u8>> {\n    tokio::task::yield_now().await;\n    let mut bytes = Vec::new();\n    file.read_to_end(&mut bytes)?;\n    Ok(bytes)\n}",
        &[5],
    );
}

#[test]
fn given_blocking_file_read_to_string_when_async_then_reports_read_call() {
    assert_code_findings(
        &BlockingCallInAsync,
        "BlockingCallInAsync",
        "use std::io::Read;\nasync fn load(mut file: std::fs::File) -> std::io::Result<String> {\n    let mut text = String::new();\n    file.read_to_string(&mut text)?;\n    Ok(text)\n}\nstruct Reader;\nimpl Reader { fn read_to_string(&self, _: &mut String) {} }\nasync fn custom(reader: Reader) {\n    reader.read_to_string(&mut String::new());\n}",
        &[4],
    );
}

#[test]
fn given_blocking_call_in_iterator_closure_when_async_then_reports_call_outside_spawn_blocking() {
    assert_code_findings(
        &BlockingCallInAsync,
        "BlockingCallInAsync",
        "async fn sizes(paths: Vec<std::path::PathBuf>) -> usize {\n    let readable = paths\n        .iter()\n        .filter_map(|path| std::fs::metadata(path).ok())\n        .count();\n    let loaded = tokio::task::spawn_blocking(|| std::fs::read(\"file\")).await;\n    readable\n}",
        &[4],
    );
}
