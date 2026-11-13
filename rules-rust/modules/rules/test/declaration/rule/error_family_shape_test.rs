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
#[test]
fn given_error_struct_or_foreign_from_type_when_checked_then_requires_enum_and_owned_family_composition()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "#[derive(Debug, thiserror::Error)] #[error(\"failed\")] struct Failure;",
        &[1],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "#[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"failed\")] Failed(#[from] vendor::Failure) }",
        &[1],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "struct Failure; #[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"failed\")] Failed(#[from] Failure) }",
        &[],
    );
}

#[test]
fn given_foreign_error_nested_in_an_option_field_when_checked_then_the_family_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "type Source = Box<dyn std::error::Error + Send + Sync>;\n#[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"failed\")] Failed { cause: Option<std::io::Error> } }\n#[derive(Debug, thiserror::Error)] enum LoadError { #[error(\"failed\")] Failed { #[source] source: Source } }",
        &[2],
    );
}

#[test]
fn given_foreign_error_type_from_an_error_module_when_checked_then_the_family_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "#[derive(Debug, thiserror::Error)] enum WaitError { #[error(\"timed out\")] Timeout(tokio::time::error::Elapsed) }\n#[derive(Debug, thiserror::Error)] enum PathError { #[error(\"missing\")] Missing { path: std::path::PathBuf } }",
        &[1],
    );
}

#[test]
fn given_own_family_as_a_source_field_when_checked_then_only_the_foreign_source_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "#[derive(Debug, thiserror::Error)] enum StorageError { #[error(\"disk\")] Disk }\n#[derive(Debug, thiserror::Error)] enum OrderError { #[error(\"load\")] Load { #[source] source: StorageError }, #[error(\"save\")] Save { #[source] source: StorageError } }\n#[derive(Debug, thiserror::Error)] enum ReadError { #[error(\"read\")] Read { #[source] source: std::io::Error } }",
        &[3],
    );
}

#[test]
fn given_field_that_is_not_send_or_sync_when_checked_then_the_family_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "#[derive(Debug, thiserror::Error)] enum NameError { #[error(\"blank\")] Blank { name: std::rc::Rc<str> } }\n#[derive(Debug, thiserror::Error)] enum ListError { #[error(\"empty\")] Empty { names: Vec<std::cell::RefCell<String>> } }\n#[derive(Debug, thiserror::Error)] enum SharedError { #[error(\"blank\")] Blank { name: std::sync::Arc<str> } }",
        &[1, 2],
    );
}

#[test]
fn given_enum_without_error_suffix_and_a_handwritten_error_impl_when_checked_then_the_family_is_checked()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ErrorFamilyShape,
        "#[derive(Debug)] enum Failure { Missing }\nimpl std::fmt::Display for Failure { fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { formatter.write_str(\"missing\") } }\nimpl std::error::Error for Failure {}\nenum Kind { Plain }",
        &[1],
    );
}
