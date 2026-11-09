#[test]
fn given_deferred_and_direct_service_fields_when_checked_then_only_arc_once_lock_or_owned_module_pass()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod api { pub trait Repo {} } mod di {\nstruct WrongModule { repo: Option<std::sync::Arc<dyn crate::api::Repo>> }\nstruct DeferredModule { repo: tokio::sync::OnceCell<std::sync::Arc<dyn crate::api::Repo>> }\nstruct CorrectModule { repo: std::sync::OnceLock<std::sync::Arc<dyn crate::api::Repo>> } impl CorrectModule { fn repo(&self) -> &std::sync::Arc<dyn crate::api::Repo> { self.repo.get_or_init(|| todo!()) } }\n}",
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
#[test]
fn given_service_storage_when_checked_then_traitless_usecases_are_shared_and_contract_services_are_dynamic()
 {
    for field in [
        "crate::usecase::ReadInvoiceUseCase",
        "std::sync::Arc<crate::file::FileRepo>",
        "std::sync::Arc<dyn crate::api::Repo>",
        "std::sync::Arc<crate::usecase::ReadInvoiceUseCase>",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleShape,
            &format!(
                "mod api {{ pub trait Repo {{}} }} mod file {{ pub struct FileRepo; impl crate::api::Repo for FileRepo {{}} }} mod usecase {{ pub struct ReadInvoiceUseCase; }} mod di {{ struct InvoiceModule {{ service: {field} }} }}"
            ),
            if field == "crate::usecase::ReadInvoiceUseCase" || field.contains("FileRepo") {
                &[1]
            } else {
                &[]
            },
        );
    }
}

#[test]
fn given_deferred_accessors_when_checked_then_get_or_init_and_delegation_pass_and_set_get_fails() {
    for body in [
        "self.repo.get().unwrap()",
        "self.repo.get_or_init(|| todo!())",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleShape,
            &format!(
                "mod di {{ struct InvoiceModule {{ repo: std::sync::OnceLock<String> }} impl InvoiceModule {{ fn repo(&self) -> &String {{ {body} }} }} }}"
            ),
            if body.contains("get_or_init") {
                &[]
            } else {
                &[1]
            },
        );
    }
}

#[test]
fn given_available_provider_module_when_checked_then_receive_module_and_preserve_contract_fallback_without_provider()
 {
    for provider in [
        "struct ProviderModule { repo: std::sync::Arc<dyn crate::api::Repo> }",
        "",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleShape,
            &format!(
                "mod api {{ pub trait Repo {{}} }} mod di {{ {provider} struct ConsumerModule; impl ConsumerModule {{ fn new(repo: std::sync::Arc<dyn crate::api::Repo>) -> Self {{ Self }} }} }}"
            ),
            if provider.is_empty() { &[] } else { &[1] },
        );
    }
}
#[test]
fn given_workspace_provider_accessibility_when_checked_then_contract_fallback_is_preserved() {
    use crate::check::api::Rule;
    for available in [false, true] {
        let dependencies = if available {
            "contract={path=\"../contract\"}\nprovider={path=\"../provider\"}"
        } else {
            "contract={path=\"../contract\"}"
        };
        let context = crate::test::module::util::read_workspace_shape_context(&[
            ("contract", "", "pub mod api { pub trait Repo {} }"),
            (
                "provider",
                "contract={path=\"../contract\"}",
                "pub mod di { pub struct ProviderModule { pub repo: std::sync::Arc<dyn contract::api::Repo> } }",
            ),
            (
                "consumer",
                dependencies,
                "mod di { struct ConsumerModule; impl ConsumerModule { fn new(repo: std::sync::Arc<dyn contract::api::Repo>) -> Self { Self } } }",
            ),
        ]);
        assert_eq!(
            crate::module::rule::DiModuleShape.check(&context).len(),
            usize::from(available)
        );
    }
}
#[test]
fn given_cfg_exclusive_provider_when_checked_then_contract_fallback_remains_available() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod api { pub trait Repo {} } mod di { #[cfg(feature=\"a\")] struct ProviderModule { repo: std::sync::Arc<dyn crate::api::Repo> } #[cfg(not(feature=\"a\"))] struct ConsumerModule; #[cfg(not(feature=\"a\"))] impl ConsumerModule { fn new(repo: std::sync::Arc<dyn crate::api::Repo>) -> Self { Self } } }",
        &[],
    );
}
#[test]
fn given_delegated_deferred_accessor_when_checked_then_initialization_chain_is_required() {
    for initialize in [
        "self.repo.get_or_init(|| String::new())",
        "self.repo.set(String::new()).unwrap(); self.repo.get().unwrap()",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleShape,
            &format!(
                "mod di {{ struct InvoiceModule {{ repo: std::sync::OnceLock<String> }} impl InvoiceModule {{ fn repo(&self) -> &String {{ self.initialize_repo() }} fn initialize_repo(&self) -> &String {{ {initialize} }} }} }}"
            ),
            if initialize.contains("get_or_init") {
                &[]
            } else {
                &[1]
            },
        );
    }
}
