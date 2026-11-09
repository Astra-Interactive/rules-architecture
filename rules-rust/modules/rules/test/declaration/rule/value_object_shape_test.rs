#[test]
fn given_validated_value_objects_when_checked_then_private_fields_accessors_and_parser_placement_are_enforced()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model { struct Money(i64); impl Money { fn new(cents: i64) -> Self { assert!(cents >= 0); Self(cents) } fn cents(&self) -> i64 { self.0 } } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model {\n struct Money(pub i64); impl Money { fn new(cents: i64) -> Self { assert!(cents >= 0); Self(cents) } }\n fn parse_money() -> Money { todo!() } }",
        &[2, 3],
    );
}

#[test]
fn given_di_alias_shape_when_checked_then_preserves_defined_exceptions() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model { struct Money(i64); impl Money { fn new(limit: i64, cents: i64) -> Self { assert!(cents <= limit); Self(cents) } fn cents(&self) -> i64 { self.0 } } }",
        &[],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::declaration::rule::ValueObjectShape,
            "ValueObjectShape",
            half,
        );
    }
}
