use crate::declaration::rule::LocalItem;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&LocalItem::new(), "LocalItem", "bad");
    assert_code_fixture(&LocalItem::new(), "LocalItem", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &LocalItem::new(),
        "LocalItem",
        "fn run() {\n    struct Value;\n    enum Choice {}\n    trait Work {}\n    impl Value {}\n    mod nested {}\n    union Bits { value: u8 }\n    consume(|| { struct Inner; });\n    spawn(async { struct AsyncValue; });\n}",
        &[2, 3, 4, 5, 6, 7, 8, 9],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &LocalItem::new(),
        "LocalItem",
        "struct Value;\nimpl Value { fn run() { let value = 1; } }\nmod nested { struct Other; }",
        &[],
    );
}
