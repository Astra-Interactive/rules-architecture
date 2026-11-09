use crate::runtime::rule::IgnoredTest;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&IgnoredTest, "IgnoredTest", "bad");
    assert_code_fixture(&IgnoredTest, "IgnoredTest", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &IgnoredTest,
        "IgnoredTest",
        "#[test]\n#[ignore]\nfn run() {}\n#[ignore = \"slow\"]\nfn other() {}",
        &[2, 4],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &IgnoredTest,
        "IgnoredTest",
        "#[test]\nfn run() {}\nfn other() {}",
        &[],
    );
}
