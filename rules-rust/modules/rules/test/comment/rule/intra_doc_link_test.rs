use crate::comment::rule::IntraDocLink;
use crate::test::declaration::util::assert_code_findings;

#[test]
fn given_resolved_function_names_with_empty_parentheses_in_code_when_checked_then_reports_only_those()
 {
    assert_code_findings(
        &IntraDocLink,
        "IntraDocLink",
        "pub fn compute_total(subtotal: u32, tax: u32) -> u32 { subtotal + tax } struct Order; impl Order { fn load() {} }\n/// Rounds the value that `compute_total()` returns.\n/// It also calls `Order::load()`.\n/// Not `compute_total(subtotal, tax)`, `vec![]` or `missing()`.\n/// Linked as [`compute_total()`] already.\npub fn round_total(total: u32) -> u32 { total }",
        &[2, 3],
    );
}
