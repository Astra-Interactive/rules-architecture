use crate::comment::rule::RustdocFiller;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&RustdocFiller, "RustdocFiller", "bad");
    assert_code_fixture(&RustdocFiller, "RustdocFiller", "good");
}

#[test]
fn given_module_doc_that_is_responsible_for_when_checked_then_reports_it_but_not_prose() {
    assert_code_findings(
        &RustdocFiller,
        "RustdocFiller",
        "mod parser {\n    //! This module is responsible for parsing invoices.\n}\nmod reader {\n    //! The module reads invoices from disk.\n}",
        &[2],
    );
}

#[test]
fn given_parameter_restatements_in_other_line_shapes_when_checked_then_reports_each_line() {
    assert_code_findings(
        &RustdocFiller,
        "RustdocFiller",
        "/// Loads the user.\n///\n/// * `user_id` — the user id\npub fn load_user(user_id: u64) {}\n/// Loads the order.\n///\n/// - order_id: the order id\npub fn load_order(order_id: u64) {}\n/// Loads the plan.\n///\n/// - `plan_id` – the plan id\npub fn load_plan(plan_id: u64) {}\n/// Computes the discount.\n///\n/// - `rate` is a fraction in the range `0.0..=1.0`\npub fn compute(rate: f64) {}",
        &[3, 7, 11],
    );
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
fn given_this_item_kind_is_responsible_for_when_checked_then_reports_each_kind_but_not_prose() {
    assert_code_findings(
        &RustdocFiller,
        "RustdocFiller",
        "/// This struct is responsible for loading users.\npub struct UserLoader;\n/// This enum is responsible for naming states.\npub enum State { Ready }\n/// This trait is responsible for reading users.\npub trait UserReader {}\n/// This type is responsible for keeping an index.\npub type Index = u32;\n/// The struct owns the connection.\npub struct Connection;\n/// This trait is implemented by every store.\npub trait Store {}\n/// This struct allows you to hold the connection.\npub struct Holder;\n/// This trait is intended for stores.\npub trait Intended {}",
        &[1, 3, 5, 7, 13, 15],
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
