use crate::comment::rule::IntraDocLink;
use crate::test::declaration::util::assert_code_findings;

#[test]
fn given_doc_naming_the_item_it_documents_when_checked_then_the_span_is_no_link_target() {
    for source in [
        "/// Ports Java's `DimensionRegistry`.\npub struct DimensionRegistry;",
        "/// Ports Java's `VersionedDimension`.\npub enum VersionedDimension { Current }",
        "/// The contract `PacketSnapshots` names.\npub trait PacketSnapshots {}",
        "/// Calls `round_total()` and `round_total` again.\npub fn round_total(total: u32) -> u32 { total }",
        "struct Order;\nimpl Order {\n    /// Same as `Order::load()`.\n    fn load() {}\n}",
    ] {
        assert_code_findings(&IntraDocLink, "IntraDocLink", source, &[]);
    }
}

#[test]
fn given_trait_method_impl_const_or_impl_type_naming_itself_when_checked_then_the_span_is_no_link_target()
 {
    for source in [
        "pub trait Store {\n    /// Same as `Store::load()`.\n    fn load(&self);\n}",
        "pub trait Store {\n    /// Same as `Self::load()`.\n    fn load(&self);\n}",
        "struct Order;\nimpl Order {\n    /// Limits `Order::LIMIT`.\n    const LIMIT: u32 = 1;\n}",
        "trait Source { type Item; }\nstruct Order;\nimpl Source for Order {\n    /// Names `Order::Item`.\n    type Item = u32;\n}",
    ] {
        assert_code_findings(&IntraDocLink, "IntraDocLink", source, &[]);
    }
}

#[test]
fn given_trait_method_impl_const_or_impl_type_naming_another_member_when_checked_then_the_span_reports()
 {
    for (source, lines) in [
        (
            "pub trait Store {\n    fn load(&self);\n    /// Same as `Store::load()`.\n    fn save(&self);\n}",
            &[3][..],
        ),
        (
            "struct Order;\nimpl Order {\n    const LIMIT: u32 = 1;\n    /// Limits `Order::LIMIT`.\n    fn cap() {}\n}",
            &[4][..],
        ),
        (
            "trait Source { type Item; fn pull(); }\nstruct Order;\nimpl Source for Order {\n    type Item = u32;\n    /// Names `Order::Item`.\n    fn pull() {}\n}",
            &[5][..],
        ),
    ] {
        assert_code_findings(&IntraDocLink, "IntraDocLink", source, lines);
    }
}

#[test]
fn given_doc_naming_another_item_when_checked_then_the_span_still_reports() {
    assert_code_findings(
        &IntraDocLink,
        "IntraDocLink",
        "pub struct Other;\n/// Mirrors `Other`.\npub struct Mine;",
        &[2],
    );
    assert_code_findings(
        &IntraDocLink,
        "IntraDocLink",
        "pub fn stop_app() {}\n/// # Safety\n///\n/// Pointers come from `stop_app`.\npub unsafe fn cleanup_token() {}",
        &[4],
    );
    assert_code_findings(
        &IntraDocLink,
        "IntraDocLink",
        "pub struct Other;\n/// Mirrors `Mine` and `Other`.\npub struct Mine;",
        &[2],
    );
}

#[test]
fn given_primitive_and_std_names_in_docs_when_checked_then_they_still_report() {
    assert_code_findings(
        &IntraDocLink,
        "IntraDocLink",
        "use std::sync::Arc;\n/// Kept as a `bool` flag.\n/// Counts in `i32`.\n/// Shared as an `Arc`.\npub struct Setting;",
        &[2, 3, 4],
    );
}

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
