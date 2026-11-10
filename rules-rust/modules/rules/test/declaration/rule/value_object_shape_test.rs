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
            "Result<crate::model::Money, crate::model::MoneyError>",
            "Ok(crate::model::Money::new(raw))",
            false,
        ),
        (
            "Result<crate::model::Money, crate::model::MoneyError>",
            "let value = crate::model::Money::new(raw); if raw < 0 { return Err(crate::model::MoneyError::Negative); } Ok(value)",
            false,
        ),
        (
            "Result<crate::model::Money, crate::model::MoneyError>",
            "if raw < 0 { return Err(crate::model::MoneyError::Negative); } Ok(crate::model::Money::new(raw))",
            true,
        ),
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::declaration::rule::ValueObjectShape,
            &format!(
                "mod model {{ pub struct Money(i64); impl Money {{ pub fn new(cents: i64) -> Self {{ assert!(cents >= 0); Self(cents) }} pub fn cents(&self) -> i64 {{ self.0 }} }} pub enum MoneyError {{ Negative }} }}\nmod util {{ fn parse_money(raw: i64) -> {signature} {{ {body} }} }}"
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

#[test]
fn given_constructor_asserting_inside_a_loop_when_checked_then_the_value_object_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model {\n#[derive(Debug, Clone, PartialEq, Eq, Hash)] pub struct TaxRates { pub percents: Vec<u8>, pub precision: u8 }\nimpl TaxRates { pub fn new(percents: Vec<u8>, precision: u8) -> Self { for percent in &percents { assert!(*percent <= 100); } Self { percents, precision } } }\n}",
        &[2],
    );
}

#[test]
fn given_parsers_rejecting_through_question_mark_or_match_arm_when_checked_then_only_the_unchecked_parser_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model {\npub struct Money(i64); impl Money { pub fn new(cents: i64) -> Self { assert!(cents >= 0); Self(cents) } pub fn cents(&self) -> i64 { self.0 } }\npub struct Quantity(i64); impl Quantity { pub fn new(count: i64) -> Self { assert!(count >= 0); Self(count) } pub fn count(&self) -> i64 { self.0 } }\npub struct Rate(i64); impl Rate { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } } pub enum MoneyError { Negative }\n}\nmod util {\nfn parse_cents(raw: i64) -> Result<i64, crate::model::MoneyError> { if raw < 0 { return Err(crate::model::MoneyError::Negative); } Ok(raw) }\nfn parse_money(raw: i64) -> Result<crate::model::Money, crate::model::MoneyError> { let cents = parse_cents(raw)?; Ok(crate::model::Money::new(cents)) }\nfn parse_quantity(raw: i64) -> Result<crate::model::Quantity, crate::model::MoneyError> { match raw { count if count >= 0 => Ok(crate::model::Quantity::new(count)), _ => Err(crate::model::MoneyError::Negative) } }\nfn parse_rate(raw: i64) -> Result<crate::model::Rate, crate::model::MoneyError> { Ok(crate::model::Rate::new(raw)) }\n}",
        &[10],
    );
}

#[test]
fn given_invariant_written_with_the_assert_family_when_checked_then_the_value_object_is_recognised()
{
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model {\npub struct Even(pub i64); impl Even { pub fn new(value: i64) -> Self { assert_eq!(value % 2, 0); Self(value) } }\npub struct Positive(pub i64); impl Positive { pub fn new(value: i64) -> Self { debug_assert!(value > 0); Self(value) } }\npub struct Small(pub i64); impl Small { pub fn new(value: i64) -> Self { std::assert!(value < 10); Self(value) } }\npub struct Distinct(pub i64); impl Distinct { pub fn new(value: i64) -> Self { assert_ne!(value, 0); Self(value) } }\npub struct Plain(pub i64); impl Plain { pub fn new(value: i64) -> Self { if value < 0 { panic!(\"negative\"); } Self(value) } }\n}",
        &[2, 3, 4, 5],
    );
}

#[test]
fn given_parsers_whose_error_is_not_a_workspace_enum_when_checked_then_reports_all_but_the_enum() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model {\npub struct Money(i64); impl Money { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub struct Quantity(i64); impl Quantity { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub struct Rate(i64); impl Rate { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub struct Level(i64); impl Level { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub struct Weight(i64); impl Weight { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub struct Price(i64); impl Price { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub enum PriceError { Negative }\n}\nmod util {\nfn parse_money(raw: i64) -> Result<crate::model::Money, String> { if raw < 0 { return Err(String::new()); } Ok(crate::model::Money::new(raw)) }\nfn parse_quantity(raw: i64) -> Result<crate::model::Quantity, &'static str> { if raw < 0 { return Err(\"negative\"); } Ok(crate::model::Quantity::new(raw)) }\nfn parse_rate(raw: i64) -> Result<crate::model::Rate, ()> { if raw < 0 { return Err(()); } Ok(crate::model::Rate::new(raw)) }\nfn parse_level(raw: i64) -> Result<crate::model::Level, u8> { if raw < 0 { return Err(0); } Ok(crate::model::Level::new(raw)) }\nfn parse_weight(raw: i64) -> Result<crate::model::Weight, std::io::Error> { if raw < 0 { return Err(std::io::Error::other(\"negative\")); } Ok(crate::model::Weight::new(raw)) }\nfn parse_price(raw: i64) -> Result<crate::model::Price, crate::model::PriceError> { if raw < 0 { return Err(crate::model::PriceError::Negative); } Ok(crate::model::Price::new(raw)) }\n}",
        &[11, 12, 13, 14, 15],
    );
}

#[test]
fn given_parsers_returning_result_aliases_when_checked_then_reports_only_the_non_enum_error() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ValueObjectShape,
        "mod model {\npub struct Money(i64); impl Money { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub struct Price(i64); impl Price { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub struct Text(i64); impl Text { pub fn new(value: i64) -> Self { assert!(value >= 0); Self(value) } pub fn value(&self) -> i64 { self.0 } }\npub enum MoneyError { Negative }\n}\nmod util {\npub type Result<T> = std::result::Result<T, crate::model::MoneyError>;\npub type MoneyParse = std::result::Result<crate::model::Price, crate::model::MoneyError>;\npub type TextResult<T> = std::result::Result<T, String>;\nfn parse_money(raw: i64) -> Result<crate::model::Money> { if raw < 0 { return Err(crate::model::MoneyError::Negative); } Ok(crate::model::Money::new(raw)) }\nfn parse_price(raw: i64) -> MoneyParse { if raw < 0 { return Err(crate::model::MoneyError::Negative); } Ok(crate::model::Price::new(raw)) }\nfn parse_text(raw: i64) -> TextResult<crate::model::Text> { if raw < 0 { return Err(String::new()); } Ok(crate::model::Text::new(raw)) }\n}",
        &[13],
    );
}
