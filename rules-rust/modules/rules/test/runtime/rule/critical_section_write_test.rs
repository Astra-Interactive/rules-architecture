use crate::runtime::rule::CriticalSectionWrite;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_option_writes_in_a_critical_section_when_checked_then_counts_them_as_writes() {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "struct Cache { slot: Option<u32>, total: u32 }\nfn run(gate: parking_lot::Mutex<Cache>) {\n    let mut guard = gate.lock();\n    guard.slot.take();\n    guard.total = 1;\n}\nfn run_once(gate: parking_lot::Mutex<Cache>) {\n    let mut guard = gate.lock();\n    let present = guard.slot.is_some();\n    guard.slot.replace(1);\n}\nfn run_option(gate: parking_lot::Mutex<Option<u32>>) {\n    let mut guard = gate.lock();\n    guard.get_or_insert(1);\n    guard.replace(2);\n}",
        &[3, 13],
    );
}

#[test]
fn given_writes_through_a_mutable_borrow_of_the_guard_when_checked_then_counts_them_as_writes() {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "struct Cache { rows: Vec<u32>, slot: Option<u32>, total: u32 }\nfn run(gate: parking_lot::Mutex<Cache>) {\n    let mut guard = gate.lock();\n    let slot = &mut guard.slot;\n    *slot = Some(1);\n    guard.total = 1;\n}\nfn run_rows(gate: parking_lot::Mutex<Cache>) {\n    let mut guard = gate.lock();\n    let rows = &mut guard.rows;\n    rows.push(1);\n    rows.push(2);\n}\nfn run_once(gate: parking_lot::Mutex<Cache>) {\n    let mut guard = gate.lock();\n    let rows = &mut guard.rows;\n    rows.push(1);\n}",
        &[3, 9],
    );
}

#[test]
fn given_nested_or_additional_collection_writes_when_checked_then_requires_last_single_write() {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<Vec<u32>>) {\n let mut guard = gate.lock(); guard.truncate(4); guard.truncate(2);\n}",
        &[2],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn side_effect() {} fn run(gate: parking_lot::Mutex<Vec<u32>>, ready: bool) {\n let mut guard = gate.lock(); if ready { guard.push(1); side_effect(); }\n}",
        &[2],
    );
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn side_effect() {} fn run(gate: parking_lot::Mutex<Vec<u32>>, ready: bool) {\n let mut guard = gate.lock(); if ready { side_effect(); guard.truncate(2); }\n}",
        &[],
    );
}

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

#[test]
fn given_critical_section_write_typed_guard_binding_when_linted_then_reports_the_local() {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "struct CachedInvoices { total: u32, revision: u32 }\nfn run(cache: parking_lot::Mutex<CachedInvoices>) {\n    let mut guard: parking_lot::MutexGuard<'_, CachedInvoices> = cache.lock();\n    guard.total = 1;\n    guard.revision = 2;\n}\nfn run_once(cache: parking_lot::Mutex<CachedInvoices>) {\n    let mut guard: parking_lot::MutexGuard<'_, CachedInvoices> = cache.lock();\n    guard.total = 1;\n}",
        &[3],
    );
}

#[test]
fn given_critical_section_write_writes_in_match_guard_when_linted_then_reports_the_local() {
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "struct CachedInvoices { total: u32, revision: u32 }\nfn run(cache: parking_lot::Mutex<CachedInvoices>, ready: bool) {\n    let mut guard = cache.lock();\n    match ready {\n        true if { guard.total = 1; guard.revision = 2; true } => {}\n        _ => {}\n    }\n}\nfn run_once(cache: parking_lot::Mutex<CachedInvoices>, ready: bool) {\n    let mut guard = cache.lock();\n    match ready {\n        true if { guard.total = 1; true } => {}\n        _ => {}\n    }\n}",
        &[3],
    );
}

#[test]
fn given_deque_ends_sorts_and_mem_swaps_under_guard_when_checked_then_counts_each_as_a_write() {
    for write in ["push_back(1)", "push_front(1)", "pop_back()", "pop_front()"] {
        assert_code_findings(
            &CriticalSectionWrite,
            "CriticalSectionWrite",
            &format!(
                "fn run(gate: parking_lot::Mutex<std::collections::VecDeque<u32>>) {{\n    let mut guard = gate.lock();\n    guard.{write};\n    guard.{write};\n}}"
            ),
            &[2],
        );
    }
    for sort in [
        "sort_by(|left, right| left.cmp(right))",
        "sort_by_key(|row| *row)",
        "sort_by_cached_key(|row| *row)",
        "sort_unstable()",
        "sort_unstable_by(|left, right| left.cmp(right))",
        "sort_unstable_by_key(|row| *row)",
    ] {
        assert_code_findings(
            &CriticalSectionWrite,
            "CriticalSectionWrite",
            &format!(
                "fn run(gate: parking_lot::Mutex<Vec<u32>>) {{\n    let mut guard = gate.lock();\n    guard.{sort};\n    guard.{sort};\n}}"
            ),
            &[2],
        );
    }
    for exchange in [
        "std::mem::take(&mut *guard)",
        "std::mem::replace(&mut *guard, Vec::new())",
        "std::mem::swap(&mut *guard, &mut spare)",
    ] {
        assert_code_findings(
            &CriticalSectionWrite,
            "CriticalSectionWrite",
            &format!(
                "fn run(gate: parking_lot::Mutex<Vec<u32>>, mut spare: Vec<u32>) {{\n    let mut guard = gate.lock();\n    {exchange};\n    {exchange};\n}}"
            ),
            &[2],
        );
    }
    assert_code_findings(
        &CriticalSectionWrite,
        "CriticalSectionWrite",
        "fn run(gate: parking_lot::Mutex<std::collections::VecDeque<u32>>) {\n    let mut guard = gate.lock();\n    guard.push_back(1);\n}",
        &[],
    );
}
