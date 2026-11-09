use crate::runtime::rule::JoinSetBehindLock;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &JoinSetBehindLock,
        "JoinSetBehindLock",
        "type Tasks = tokio::task::JoinSet<()>;\nstruct Shared { tasks: tokio::sync::Mutex<Tasks> }\n",
        &[2],
    );
    assert_code_findings(
        &JoinSetBehindLock,
        "JoinSetBehindLock",
        "struct JoinSet;\nstruct Shared { tasks: tokio::sync::Mutex<JoinSet> }\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&JoinSetBehindLock, "JoinSetBehindLock", "bad");
    assert_code_fixture(&JoinSetBehindLock, "JoinSetBehindLock", "good");
}
