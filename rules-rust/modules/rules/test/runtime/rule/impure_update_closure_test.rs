use crate::runtime::rule::ImpureUpdateClosure;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_resolved_rwlock_or_captured_collection_when_updated_then_reports_effects_only_for_known_receivers()
 {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>, gate: parking_lot::RwLock<u32>) {\n state.send_modify(|current| { let guard = gate.read(); *current = *guard; });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(counter: std::sync::atomic::AtomicUsize, captured: &mut Vec<usize>) {\n counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { captured.push(current); Some(current) });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct Buffer; impl Buffer { fn push(&self, _: usize) {} } fn run(counter: std::sync::atomic::AtomicUsize, captured: Buffer) {\n counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { captured.push(current); Some(current) });\n}",
        &[],
    );
}

#[test]
fn given_atomic_io_and_watch_capture_when_checked_then_keeps_distinct_purity_contracts() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(counter: &std::sync::atomic::AtomicUsize) {\n    counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { let content = std::fs::read(\"file\"); Some(current + 1) });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>, other: tokio::sync::mpsc::Sender<u32>) {\n    state.send_modify(|current| { let sent = other.try_send(*current); });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct Sender;\nimpl Sender { fn send_modify<F>(&self, _: F) {} }\nfn run(state: Sender) { state.send_modify(|| { let content = std::fs::read(\"file\"); }); }",
        &[],
    );
}

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(counter: &std::sync::atomic::AtomicUsize) {\n    let mut captured = 0;\n    counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { captured = current; Some(current + 1) });\n}\n",
        &[3],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>) {\n    let mut captured = None;\n    state.send_modify(|current| { captured = Some(*current); *current += 1; });\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ImpureUpdateClosure, "ImpureUpdateClosure", "bad");
    assert_code_fixture(&ImpureUpdateClosure, "ImpureUpdateClosure", "good");
}
