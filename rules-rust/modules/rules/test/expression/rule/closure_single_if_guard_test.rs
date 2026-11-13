use crate::expression::rule::ClosureSingleIfGuard;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&ClosureSingleIfGuard, "ClosureSingleIfGuard", "bad");
    assert_code_fixture(&ClosureSingleIfGuard, "ClosureSingleIfGuard", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &ClosureSingleIfGuard,
        "ClosureSingleIfGuard",
        "fn run() {\n    spawn(async {\n        if ready { first(); second(); }\n    });\n    consume(|| { if ready { first(); second(); } });\n    consume(|| if ready { first(); second(); });\n}",
        &[3, 5, 6],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &ClosureSingleIfGuard,
        "ClosureSingleIfGuard",
        "fn run() {\n    spawn(async { if ready { first(); } });\n    consume(|| { if ready { first(); second(); } else { other(); } });\n    consume(|| { if !ready { return; } first(); second(); });\n}",
        &[],
    );
}
