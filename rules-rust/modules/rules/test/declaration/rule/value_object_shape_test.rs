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
#[test]
fn given_value_object_parser_when_checked_then_result_and_validation_precede_construction() {
    for (signature, body, valid) in [
        (
            "crate::model::Money",
            "crate::model::Money::new(raw)",
            false,
        ),
        (
            "Result<crate::model::Money, ()>",
            "Ok(crate::model::Money::new(raw))",
            false,
        ),
        (
            "Result<crate::model::Money, ()>",
            "let value = crate::model::Money::new(raw); if raw < 0 { return Err(()); } Ok(value)",
            false,
        ),
        (
            "Result<crate::model::Money, ()>",
            "if raw < 0 { return Err(()); } Ok(crate::model::Money::new(raw))",
            true,
        ),
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::declaration::rule::ValueObjectShape,
            &format!(
                "mod model {{ pub struct Money(i64); impl Money {{ pub fn new(cents: i64) -> Self {{ assert!(cents >= 0); Self(cents) }} pub fn cents(&self) -> i64 {{ self.0 }} }} }}\nmod util {{ fn parse_money(raw: i64) -> {signature} {{ {body} }} }}"
            ),
            if valid { &[] } else { &[2] },
        );
    }
}
#[test]
fn given_an_accessor_that_returns_another_value_when_checked_then_the_value_object_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model { struct Money(i64); impl Money { fn new(cents: i64) -> Self { assert!(cents >= 0); Self(cents) } fn cents(&self) -> i64 { 0 } } }",
        &[1],
    );
}
