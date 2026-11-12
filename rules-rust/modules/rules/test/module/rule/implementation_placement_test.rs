use crate::module::rule::ImplementationPlacement;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

#[test]
fn given_technology_struct_implementing_a_trait_of_another_concept_when_checked_then_reports_only_that_struct()
 {
    let context = read_placement_context(&[(
        "src/lib.rs",
        "mod account { mod api { trait Reader {} } mod file { struct FileReader; impl crate::account::api::Reader for FileReader {} } }\nmod billing { mod file { struct BillingReader; impl crate::account::api::Reader for BillingReader {} } }",
    )]);
    assert_placement_findings(
        &ImplementationPlacement,
        "ImplementationPlacement",
        &context,
        &[("src/lib.rs", 2, "Node")],
    );
}

#[test]
fn given_fake_of_another_concept_when_checked_then_leaves_it_to_the_fake_rule() {
    let context = read_placement_context(&[(
        "src/lib.rs",
        "mod account { mod api { trait Reader {} } }\n#[cfg(test)] mod test { mod billing { mod fake { pub(crate) struct FakeReader; impl crate::account::api::Reader for FakeReader {} } } }",
    )]);
    assert_placement_findings(
        &ImplementationPlacement,
        "ImplementationPlacement",
        &context,
        &[],
    );
}

#[test]
fn given_technology_struct_implementing_a_non_workspace_trait_when_checked_then_reports_nothing() {
    let context = read_placement_context(&[(
        "src/lib.rs",
        "mod billing { mod file { struct BillingReader; impl std::default::Default for BillingReader { fn default() -> Self { Self } } } }",
    )]);
    assert_placement_findings(
        &ImplementationPlacement,
        "ImplementationPlacement",
        &context,
        &[],
    );
}
