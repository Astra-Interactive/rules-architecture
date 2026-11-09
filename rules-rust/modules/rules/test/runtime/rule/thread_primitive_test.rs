use crate::runtime::rule::ThreadPrimitive;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ThreadPrimitive,
        "ThreadPrimitive",
        "#[expect(clippy::disallowed_methods, reason = \"wrong boundary\")]\nfn run() { let thread = std::thread::spawn(|| {}); }\n",
        &[2],
    );
    assert_code_findings(
        &ThreadPrimitive,
        "ThreadPrimitive",
        "fn run() { let thread = std::thread::spawn(|| {}); }\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ThreadPrimitive, "ThreadPrimitive", "bad");
    assert_code_fixture(&ThreadPrimitive, "ThreadPrimitive", "good");
}
