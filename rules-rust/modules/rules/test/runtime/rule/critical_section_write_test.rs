use crate::runtime::rule::CriticalSectionWrite;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_guarded_struct_fields_when_checked_then_counts_only_resolved_collection_writes() {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "struct Cache { rows: Vec<u32>, generation: u32 }\nfn run(gate: std::sync::Mutex<Cache>) {\n    let mut guard = gate.lock().unwrap_or_else(std::sync::PoisonError::into_inner);\n    guard.rows.push(1);\n    guard.generation = 1;\n}",
        &[3],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "struct Cache { rows: Vec<u32>, generation: u32 }\nfn run(gate: parking_lot::Mutex<Cache>) {\n    let mut guard = gate.lock();\n    guard.rows.push(1);\n    guard.generation = 1;\n}",
        &[3],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "struct Buffer<T>(T);\nimpl<T> Buffer<T> { fn push(&self, _: u32) {} }\nfn run(gate: parking_lot::Mutex<Buffer<Vec<u32>>>) {\n    let guard = gate.lock();\n    guard.push(1);\n    guard.push(2);\n}",
        &[],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<Vec<u32>>, choose: bool) {\n    let mut guard = gate.lock();\n    if choose { guard.push(1); } else { guard.push(2); }\n}",
        &[],
    );
}

#[test]
fn given_write_followed_by_returned_value_when_checked_then_does_not_treat_the_tail_value_as_an_operation()
 {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<u32>) -> u32 {\n    let mut guard = gate.lock();\n    let next = 1;\n    *guard = next;\n    next\n}",
        &[],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<u32>) -> u32 {\n    let mut guard = gate.lock();\n    let next = 1;\n    *guard = next;\n    return next;\n}",
        &[],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<u32>) -> u32 {\n    let mut guard = gate.lock();\n    let next = 1;\n    *guard = next;\n    return next.clone();\n}",
        &[2],
    );
}

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<Vec<u32>>) {\n    let mut guard = gate.lock();\n    guard.push(1);\n    guard.push(2);\n}\n",
        &[2],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<Vec<u32>>) {\n    let mut guard = gate.lock();\n    guard.push(1);\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&CriticalSectionWrite, "CriticalSectionWrite", "bad");
    assert_code_fixture(&CriticalSectionWrite, "CriticalSectionWrite", "good");
}
