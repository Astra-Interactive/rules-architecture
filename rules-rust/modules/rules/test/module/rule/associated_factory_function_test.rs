use crate::module::rule::AssociatedFactoryFunction;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

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
