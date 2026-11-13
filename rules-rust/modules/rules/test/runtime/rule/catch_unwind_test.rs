use crate::runtime::rule::CatchUnwind;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_self_path_call_inside_a_caught_closure_when_checked_then_reports_it_as_own_logic() {
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "struct Parser;\nimpl Parser {\n    fn parse(raw: &str) -> u32 { raw.len() as u32 }\n    fn run(raw: &str) {\n        let parsed = std::panic::catch_unwind(|| Self::parse(raw));\n        let foreign = std::panic::catch_unwind(|| vendor::parse(raw));\n    }\n}",
        &[5],
    );
}

#[test]
fn given_owned_inline_or_method_logic_when_caught_then_reports_local_logic() {
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "fn run(value: u32) {\n let result = std::panic::catch_unwind(|| value + 1);\n}",
        &[2],
    );
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "struct Service; impl Service { fn compute(&self) {} fn run(&self) {\n let result = std::panic::catch_unwind(|| self.compute());\n} }",
        &[2],
    );
}

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "fn local() {}\nfn run() { let result = std::panic::catch_unwind(|| local()); }\n",
        &[2],
    );
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "fn run() { let result = std::panic::catch_unwind(|| std::fs::read(\"file\")); }\n",
        &[],
    );
}

#[test]
fn given_own_logic_behind_unwind_safe_wrapper_when_caught_then_reports_local_logic() {
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "struct Worker;\nimpl Worker {\n    fn compute(&self) -> u32 { 1 }\n}\nfn run(worker: &Worker) -> bool {\n    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker.compute())).is_err()\n}\nfn read() {\n    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| std::fs::read(\"file\")));\n}",
        &[6],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&CatchUnwind, "CatchUnwind", "bad");
    assert_code_fixture(&CatchUnwind, "CatchUnwind", "good");
}

#[test]
fn given_workspace_function_path_when_caught_then_reports_local_logic() {
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "fn run_batch() -> u32 { 1 }\npub(crate) fn load_named() -> bool {\n    std::panic::catch_unwind(run_batch).is_err()\n}\npub(crate) fn load_wrapped() -> bool {\n    std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_batch)).is_err()\n}\npub(crate) fn load_foreign() -> bool {\n    std::panic::catch_unwind(std::env::current_dir).is_err()\n}",
        &[3, 6],
    );
}

#[test]
fn given_closure_bound_to_a_local_when_caught_then_reports_only_the_one_running_own_logic() {
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "fn compute_total(raw: &str) -> usize { raw.len() }\npub(crate) fn run(raw: &str) -> bool {\n    let work = || compute_total(raw);\n    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err()\n}\npub(crate) fn read() -> bool {\n    let load = || std::fs::read(\"file\");\n    std::panic::catch_unwind(load).is_err()\n}\npub(crate) fn shadowed() -> bool {\n    let compute_total = |text: &str| std::fs::read(text);\n    std::panic::catch_unwind(|| compute_total(\"file\")).is_err()\n}",
        &[4],
    );
}

#[test]
fn given_caught_closure_calling_local_closures_when_checked_then_follows_the_chain_to_own_logic() {
    assert_code_findings(
        &CatchUnwind,
        "CatchUnwind",
        "fn compute_total(raw: &str) -> usize { raw.len() }\npub(crate) fn run(raw: &str) -> bool {\n    let step = || compute_total(raw);\n    let work = || step();\n    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err()\n}\npub(crate) fn run_deep(raw: &str) -> bool {\n    let inner = || compute_total(raw);\n    let middle = || inner();\n    let outer = || middle();\n    std::panic::catch_unwind(outer).is_err()\n}\npub(crate) fn read() -> bool {\n    let load = || std::fs::read(\"file\");\n    let work = || load();\n    std::panic::catch_unwind(work).is_err()\n}\npub(crate) fn shadowed_chain() -> bool {\n    let fetch = || std::fs::read(\"file\");\n    let fetch = || fetch();\n    std::panic::catch_unwind(fetch).is_err()\n}",
        &[5, 11],
    );
}
