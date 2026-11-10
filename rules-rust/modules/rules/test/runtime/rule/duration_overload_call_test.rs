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

#[test]
fn given_duration_overload_call_arithmetic_roundtrip_when_linted_then_reports_the_call() {
    assert_code_findings(
        &DurationOverloadCall,
        "DurationOverloadCall",
        "fn delay(delay: std::time::Duration) -> std::time::Duration {\n    std::time::Duration::from_millis(delay.as_secs() * 1000)\n}\nfn fixed() -> std::time::Duration {\n    std::time::Duration::from_millis(2 * 1000)\n}",
        &[2],
    );
}
