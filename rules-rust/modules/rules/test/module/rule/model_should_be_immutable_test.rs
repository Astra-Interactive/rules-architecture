use crate::check::api::Rule;
use crate::module::rule::ModelShouldBeImmutable;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::assert_placement_findings;

#[test]
fn given_an_external_receiver_sharing_a_unique_model_field_when_checked_then_external_assignment_does_not_mutate_model()
 {
    let context = crate::test::source::util::read_resolution_context();
    assert!(
        context
            .indexes
            .iter()
            .any(|index| index.items.iter().any(|item| item.name == "Snapshot"))
    );
    assert!(ModelShouldBeImmutable.check(&context).is_empty());
}

#[test]
fn given_aliased_typed_mutable_self_and_unrelated_type_when_checked_then_reports_only_the_live_model_method()
 {
    let context = read_code_context(
        "mod model { struct Value; }\nmod file { struct Value; impl Value { fn change(&mut self) {} } }\nmod util { type Alias = crate::model::Value; impl Alias { fn consume(mut self) -> Self { self } fn change(self: &mut Self) {} #[cfg(all(target_os = \"linux\", target_os = \"windows\"))] fn inactive(&mut self) {} } }",
    );
    assert_placement_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        &context,
        &[("src/lib.rs", 3, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ModelShouldBeImmutable, "ModelShouldBeImmutable", "bad");
    assert_code_fixture(&ModelShouldBeImmutable, "ModelShouldBeImmutable", "good");
}
