use crate::runtime::rule::TimePrimitive;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "use std::time::Instant as Tick;\nstruct Stored { at: Tick }\nfn now() -> Tick { loop {} }\nconst DELAY_MS: u64 = 40;\n",
        &[2, 3, 4],
    );
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "use std::time::Duration;\nfn decode_timeout(timeout_ms: u64) -> Duration { Duration::from_millis(timeout_ms) }\nconst DELAY: Duration = Duration::from_millis(40);\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&TimePrimitive, "TimePrimitive", "bad");
    assert_code_fixture(&TimePrimitive, "TimePrimitive", "good");
}
