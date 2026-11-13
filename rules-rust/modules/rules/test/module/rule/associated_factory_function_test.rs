use crate::check::api::Rule;
use crate::module::rule::AssociatedFactoryFunction;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::module::util::read_workspace_shape_context;

#[test]
fn given_non_factory_helpers_when_named_create_then_does_not_infer_an_aggregate_from_name() {
    assert_code_findings(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "mod test { struct Store; fn create_store() -> Store { Store } }\nfn create_message() -> String { String::new() }",
        &[],
    );
}

#[test]
fn given_result_alias_or_unrelated_result_type_when_checked_then_uses_resolved_return_identity() {
    assert_code_findings(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "mod model { pub struct Value; } mod util { type Outcome = Result<crate::model::Value, ()>; fn create_value() -> Outcome { Ok(crate::model::Value) } }",
        &[],
    );
    assert_code_findings(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "mod model { pub struct Result; } mod util { use crate::model::Result; fn create_value() -> Result { Result } }",
        &[1],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "bad",
    );
    assert_code_fixture(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "good",
    );
}

#[test]
fn given_create_function_returning_foreign_crate_model_when_checked_then_reports_factory() {
    let context = read_workspace_shape_context(&[
        (
            "shop_billing_api",
            "",
            "pub mod model { pub struct Invoice { pub total: u64, pub paid_at: Option<std::time::SystemTime> } }",
        ),
        (
            "shop_billing_impl",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            "mod util {\n\
             use shop_billing_api::model::Invoice;\n\
             fn create_invoice(total: u64) -> Invoice {\n\
             Invoice { total, paid_at: None }\n\
             }\n\
             fn create_checked_invoice(total: u64) -> Result<Invoice, ()> {\n\
             Ok(Invoice { total, paid_at: None })\n\
             }\n\
             }",
        ),
    ]);
    let lines: Vec<Option<u32>> = AssociatedFactoryFunction
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(3)], lines);
}

#[test]
fn given_associated_new_returning_result_when_checked_then_only_the_fallible_one_reports() {
    assert_code_findings(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "mod model {\npub struct Invoice; pub enum InvoiceError { Empty }\nimpl Invoice { fn new(lines: u32) -> Result<Invoice, InvoiceError> { Ok(Invoice) } }\npub struct Money(u64);\nimpl Money { fn new(cents: u64) -> Self { assert!(cents > 0); Self(cents) } }\n}",
        &[3],
    );
}
