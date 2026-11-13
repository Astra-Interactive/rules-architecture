use crate::module::rule::ModelContainsFunctions;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

#[test]
fn given_cfg_disjoint_impls_when_checked_then_counts_only_coexisting_impls_and_invariants() {
    let context = read_code_context(
        "mod model {\nstruct Value { value: u64 }\n#[cfg(feature = \"checked\")] impl Value { fn new(value: u64) -> Self { assert!(value > 0); Self { value } } fn value(&self) -> u64 { self.value } }\n#[cfg(not(feature = \"checked\"))] impl Value { const LIMIT: u64 = 1; }\n}",
    );
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[],
    );
    let context = read_code_context(
        "mod model {\nstruct Value { value: u64 }\n#[cfg(feature = \"checked\")] impl Value { fn new(value: u64) -> Self { assert!(value > 0); Self { value } } }\n#[cfg(not(feature = \"checked\"))] impl Value { fn value(&self) -> u64 { self.value } }\n}",
    );
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[("src/lib.rs", 4, "Node")],
    );
}

#[test]
fn given_rule_in_util_file_and_trait_impl_in_model_file_when_checked_then_preserves_five_exception()
{
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod model { pub(crate) use value::Value; mod value; } mod util { mod value_ext; }",
        ),
        (
            "src/model/value.rs",
            "pub(crate) struct Value; impl std::fmt::Display for Value {}",
        ),
        (
            "src/util/value_ext.rs",
            "type Alias = crate::model::Value; impl Alias { fn compute(&self) -> u64 { 1 } }",
        ),
    ]);
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[],
    );
}

#[test]
fn given_accessor_adapting_its_field_by_a_borrow_method_when_checked_then_only_the_rule_reports() {
    let context = read_code_context(
        "mod model {\nstruct CustomerNote(Option<String>);\nimpl CustomerNote {\nfn new(value: Option<String>) -> Self { assert!(value.is_some()); Self(value) }\nfn value(&self) -> Option<&str> { self.0.as_deref() }\nfn label(&self) -> String { self.0.clone().unwrap_or_default() }\n}\n}",
    );
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[("src/lib.rs", 6, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ModelContainsFunctions, "ModelContainsFunctions", "bad");
    assert_code_fixture(&ModelContainsFunctions, "ModelContainsFunctions", "good");
}

#[test]
fn given_type_with_try_from_in_its_file_when_checked_then_new_and_accessors_are_allowed_with_or_without_assert()
 {
    let context = read_code_context(
        "mod model {\nstruct Money(i64);\nimpl Money { fn new(cents: i64) -> Self { Self::try_from(cents).unwrap_or_else(|_| panic!(\"negative\")) } fn cents(&self) -> i64 { self.0 } }\nimpl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }\nstruct Level(u8);\nimpl Level { fn new(value: u8) -> Self { assert!(value < 10); Self(value) } fn value(&self) -> u8 { self.0 } }\nimpl std::convert::TryFrom<u8> for Level { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }\n}",
    );
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[],
    );
}

#[test]
fn given_asserting_new_and_accessors_without_try_from_when_checked_then_the_form_stays_allowed() {
    let context = read_code_context(
        "mod model {\nstruct Value { field: u64 }\nimpl Value { fn new(field: u64) -> Self { assert!(field > 0); Self { field } } fn field(&self) -> u64 { self.field } const LIMIT: u64 = 1; }\n}",
    );
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[],
    );
}

#[test]
fn given_new_and_accessor_without_assert_or_try_from_in_the_file_when_checked_then_each_is_reported()
 {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod model { pub(crate) use money::Money; mod money; mod money_conversion; }",
        ),
        (
            "src/model/money.rs",
            "pub(crate) struct Money(i64);\nimpl Money {\npub(crate) fn new(cents: i64) -> Self { Self(cents) }\npub(crate) fn cents(&self) -> i64 { self.0 }\n}",
        ),
        (
            "src/model/money_conversion.rs",
            "use crate::model::Money;\nimpl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }",
        ),
    ]);
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[
            ("src/model/money.rs", 3, "Node"),
            ("src/model/money.rs", 4, "Node"),
        ],
    );
}

#[test]
fn given_other_function_in_the_inherent_impl_of_a_try_from_type_when_checked_then_it_is_reported() {
    let context = read_code_context(
        "mod model {\nstruct Money(i64);\nimpl Money {\nfn cents(&self) -> i64 { self.0 }\nfn is_zero(&self) -> bool { self.0 == 0 }\nfn from_text(text: &str) -> Self { Self(0) }\n}\nimpl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }\n}",
    );
    assert_placement_findings(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        &context,
        &[("src/lib.rs", 5, "Node"), ("src/lib.rs", 6, "Node")],
    );
}

#[test]
fn given_try_from_fixtures_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        "try-from-bad",
    );
    assert_code_fixture(
        &ModelContainsFunctions,
        "ModelContainsFunctions",
        "try-from-good",
    );
}
