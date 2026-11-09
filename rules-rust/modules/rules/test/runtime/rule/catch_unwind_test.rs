use crate::runtime::rule::CatchUnwind;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

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
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&CatchUnwind, "CatchUnwind", "bad");
    assert_code_fixture(&CatchUnwind, "CatchUnwind", "good");
}
