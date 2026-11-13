use crate::check::api::Rule;
use crate::runtime::rule::BridgeCleanup;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};

#[test]
fn given_free_executor_block_on_in_drop_when_checked_then_reports_only_that_call() {
    assert_code_findings(
        &BridgeCleanup,
        "BridgeCleanup",
        "struct Session { client: u32 }\nimpl Drop for Session {\n    fn drop(&mut self) {\n        futures::executor::block_on(async {});\n    }\n}\nimpl Session {\n    fn close(&self) { futures::executor::block_on(async {}); }\n}\nfn block_on<T>(value: T) -> T { value }\nstruct Local;\nimpl Drop for Local {\n    fn drop(&mut self) { block_on(1); }\n}",
        &[4],
    );
}

#[test]
fn given_async_ownership_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &BridgeCleanup,
        "BridgeCleanup",
        "struct Guard;\nimpl Drop for Guard {\n    fn drop(&mut self) { tokio::spawn(async {}); }\n}\n",
        &[3],
    );
    assert_code_findings(
        &BridgeCleanup,
        "BridgeCleanup",
        "struct Guard;\nimpl Guard { fn drop(&mut self) { let handle = tokio::spawn(async {}); } }\n",
        &[],
    );
}

#[test]
fn given_persistent_cases_when_checked_then_rejects_unexpected_unresolved_source() {
    let source = "struct Guard;\nimpl Drop for Guard {\n    fn drop(&mut self) { tokio::spawn(async {}); }\n}\n";
    let context = read_code_context(source);
    for index in &context.indexes {
        assert!(index.unanalysable.is_empty(), "{:?}", index.unanalysable);
    }
    assert_eq!(1, BridgeCleanup.check(&context).len());
}

#[test]
fn given_blocking_wait_in_drop_when_checked_then_reports_block_on_and_sleep_only_there() {
    assert_code_findings(
        &BridgeCleanup,
        "BridgeCleanup",
        "struct Registration { handle: tokio::runtime::Handle }\nimpl Drop for Registration {\n    fn drop(&mut self) {\n        self.handle.block_on(async {});\n    }\n}\nstruct Pause;\nimpl Drop for Pause {\n    fn drop(&mut self) {\n        std::thread::sleep(std::time::Duration::from_millis(1));\n    }\n}\nstruct Queue;\nimpl Queue { fn block_on(&self, _: u32) {} }\nstruct Flush { queue: Queue }\nimpl Drop for Flush {\n    fn drop(&mut self) { self.queue.block_on(1); }\n}\nstruct Teardown { handle: tokio::runtime::Handle }\nimpl Teardown {\n    fn close(&self) { self.handle.block_on(async {}); }\n}",
        &[4, 10],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&BridgeCleanup, "BridgeCleanup", "bad");
    assert_code_fixture(&BridgeCleanup, "BridgeCleanup", "good");
}
