#[test]
fn given_error_source_aliases_when_checked_then_boxed_standard_error_is_accepted_and_nested_boxes_are_rejected()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "type Source = Box<dyn std::error::Error + Send + Sync>; #[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"failed\")] Failed { #[source] source: Source } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "#[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"failed\")] Failed { #[source] source: std::io::Error } }",
        &[1],
    );
}

#[test]
fn given_generic_di_aliases_when_checked_then_substituted_contract_types_are_used() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "type Source<E: ?Sized> = Box<E>; #[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"failed\")] Failed { #[source] source: Source<dyn std::error::Error + Send + Sync> } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "type Source<E> = Box<E>; #[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"failed\")] Failed { #[source] source: Source<std::io::Error> } }",
        &[1],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::declaration::rule::ErrorFamilyShape,
            "ErrorFamilyShape",
            half,
        );
    }
}
