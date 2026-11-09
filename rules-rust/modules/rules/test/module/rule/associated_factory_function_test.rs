use crate::module::rule::AssociatedFactoryFunction;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_result_alias_or_unrelated_result_type_when_checked_then_uses_resolved_return_identity() {
    assert_code_findings(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "mod util { type Outcome = Result<(), ()>; fn create_value() -> Outcome { Ok(()) } }",
        &[],
    );
    assert_code_findings(
        &AssociatedFactoryFunction,
        "AssociatedFactoryFunction",
        "mod util { struct Result; fn create_value() -> Result { Result } }",
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
