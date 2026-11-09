use crate::check::api::Rule;
use crate::runtime::rule::BridgeCleanup;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};

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
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&BridgeCleanup, "BridgeCleanup", "bad");
    assert_code_fixture(&BridgeCleanup, "BridgeCleanup", "good");
}
