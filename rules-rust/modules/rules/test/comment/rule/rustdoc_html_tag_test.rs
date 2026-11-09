use crate::comment::rule::RustdocHtmlTag;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&RustdocHtmlTag, "RustdocHtmlTag", "bad");
    assert_code_fixture(&RustdocHtmlTag, "RustdocHtmlTag", "good");
}

#[test]
fn given_unclosed_example_on_another_item_when_checked_then_fence_does_not_hide_html() {
    assert_code_findings(
        &RustdocHtmlTag,
        "RustdocHtmlTag",
        "/// ```html\n/// <p>example</p>\nfn sample() {}\n/// <p>Invalid</p>\nfn run() {}",
        &[4],
    );
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &RustdocHtmlTag,
        "RustdocHtmlTag",
        "/// <p>Invalid</p>\nfn run() {}\n/// <code class=\"x\">Invalid</code>\nstruct Value;",
        &[1, 3],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &RustdocHtmlTag,
        "RustdocHtmlTag",
        "/// Use `<p>` and `<code>` literally.\n/// ```html\n/// <p>Code</p>\n/// ```\n/// ~~~html\n/// <br>\n/// ~~~\nfn run() {}\n/// <https://example.org>\nstruct Value;",
        &[],
    );
}
