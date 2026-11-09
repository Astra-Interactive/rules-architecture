use crate::comment::rule::RustdocFiller;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&RustdocFiller, "RustdocFiller", "bad");
    assert_code_fixture(&RustdocFiller, "RustdocFiller", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &RustdocFiller,
        "RustdocFiller",
        "/// This function is intended for loading a user.\nfn load_user() {}\n/// Returns the result.\nfn load_result() {}\n/// First. Second. Third. Fourth. Fifth.\nfn load_many() {}",
        &[1, 3, 5],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &RustdocFiller,
        "RustdocFiller",
        "/// # Errors\n///\n/// Returns [`Error::Missing`] for deleted users.\nfn load_user() {}\n/// Computes the discount, excluding promotional items.\n///\n/// The rate is a fraction, rounded half up.\nfn compute_discount() {}",
        &[],
    );
}
