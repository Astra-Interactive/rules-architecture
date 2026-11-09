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
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ModelContainsFunctions, "ModelContainsFunctions", "bad");
    assert_code_fixture(&ModelContainsFunctions, "ModelContainsFunctions", "good");
}
