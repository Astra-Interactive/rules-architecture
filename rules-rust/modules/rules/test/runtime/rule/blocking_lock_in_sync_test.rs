use crate::runtime::rule::BlockingLockInSync;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_unrelated_generic_container_and_shadowed_names_when_checked_then_keeps_type_identity() {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Container<T>(T);\nimpl<T> Container<T> { fn blocking_lock(&self) {} }\nfn on_disable(gate: Container<tokio::sync::Mutex<()>>) { gate.blocking_lock(); }",
        &[],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Local;\nimpl Local { fn blocking_lock(&self) {} }\nfn on_disable(gate: tokio::sync::Mutex<()>) { let gate = Local; gate.blocking_lock(); }",
        &[],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "#[cfg(unix)] type Gate = Local;\n#[cfg(not(unix))] type Gate = tokio::sync::Mutex<()>;\nstruct Local;\nimpl Local { fn blocking_lock(&self) {} }\n#[cfg(unix)] fn on_disable(gate: Gate) { gate.blocking_lock(); }",
        &[],
    );
}

#[test]
fn given_typed_runtime_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "type Gate = tokio::sync::Mutex<u32>;\nstruct Owner { gate: Gate }\nimpl Owner { fn on_disable(&self) { self.gate.blocking_lock(); } }\n",
        &[3],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Mutex;\nimpl Mutex { fn blocking_lock(&self) {} }\nstruct Owner { gate: Mutex }\nimpl Owner { fn on_disable(&self) { self.gate.blocking_lock(); } }\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&BlockingLockInSync, "BlockingLockInSync", "bad");
    assert_code_fixture(&BlockingLockInSync, "BlockingLockInSync", "good");
}
