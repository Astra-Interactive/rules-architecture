#[test]
fn given_deferred_and_direct_service_fields_when_checked_then_only_arc_once_lock_or_owned_module_pass()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod api { pub trait Repo {} } mod di {\nstruct WrongModule { repo: Option<std::sync::Arc<dyn crate::api::Repo>> }\nstruct DeferredModule { repo: tokio::sync::OnceCell<std::sync::Arc<dyn crate::api::Repo>> }\nstruct CorrectModule { repo: std::sync::OnceLock<std::sync::Arc<dyn crate::api::Repo>> }\n}",
        &[2, 3],
    );
}

#[test]
fn given_workspace_contract_generics_when_di_shape_checked_then_trait_objects_and_unrelated_generics_pass()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod api { pub trait Repo {} } mod di { struct InvoiceModule<T: crate::api::Repo>(T); }",
        &[1],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod api { pub trait Repo {} } mod di { struct InvoiceModule { repo: std::sync::Arc<dyn crate::api::Repo> } struct OtherModule<T>(T); }",
        &[],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::DiModuleShape,
            "DiModuleShape",
            half,
        );
    }
}
