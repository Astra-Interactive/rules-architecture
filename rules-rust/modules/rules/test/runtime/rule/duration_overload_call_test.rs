use crate::runtime::rule::DurationOverloadCall;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_typed_runtime_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &DurationOverloadCall,
        "DurationOverloadCall",
        "use std::time::Duration as Delay;\nfn run(delay: Delay) {\n    let copy = Delay::from_secs(delay.as_secs());\n}\n",
        &[3],
    );
    assert_code_findings(
        &DurationOverloadCall,
        "DurationOverloadCall",
        "struct Duration;\nimpl Duration { fn from_millis(_: u128) -> Self { Self } fn as_millis(&self) -> u128 { 0 } }\nfn run(delay: Duration) { let copy = Duration::from_millis(delay.as_millis()); }\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&DurationOverloadCall, "DurationOverloadCall", "bad");
    assert_code_fixture(&DurationOverloadCall, "DurationOverloadCall", "good");
}
