use crate::expression::rule::BooleanOperatorWrapping;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&BooleanOperatorWrapping, "BooleanOperatorWrapping", "bad");
    assert_code_fixture(&BooleanOperatorWrapping, "BooleanOperatorWrapping", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &BooleanOperatorWrapping,
        "BooleanOperatorWrapping",
        "fn run() {\n    if ready\n        && current.is_valid() {}\n    while ready\n        || attempts > 3 {}\n    match value {\n        item if ready\n            && item.is_valid() => {},\n        _ => {}\n    }\n}",
        &[2, 4, 7],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &BooleanOperatorWrapping,
        "BooleanOperatorWrapping",
        "fn run() {\n    if ready\n        && !busy\n        && true {}\n    if ready && current.is_valid() {}\n    let condition = ready\n        && current.is_valid();\n}",
        &[],
    );
}
