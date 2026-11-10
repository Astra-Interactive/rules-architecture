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

#[test]
fn given_one_doc_line_for_every_parameter_when_checked_then_reports_the_comment() {
    assert_code_findings(
        &RustdocFiller,
        "RustdocFiller",
        "/// Computes the discount amount.\n///\n/// - `price` — amount in cents before tax\n/// - `rate` — fraction in the range `0.0..=1.0`\npub fn compute_discount(price: u64, rate: f64) -> u64 {\n    price\n}\n/// Computes the discount amount.\n///\n/// The rate is a fraction in the range `0.0..=1.0`.\npub fn compute_rate(price: u64, rate: f64) -> u64 {\n    price\n}",
        &[1],
    );
}
