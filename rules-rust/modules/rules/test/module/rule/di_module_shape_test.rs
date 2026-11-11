#[test]
fn given_traitless_behavior_storage_when_checked_then_shared_instance_is_required() {
    for field in [
        "crate::SmtpMailSender",
        "std::sync::Arc<crate::SmtpMailSender>",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleShape,
            &format!(
                "struct SmtpMailSender; impl SmtpMailSender {{ fn send(&self) {{ let _result = std::net::TcpStream::connect(\"localhost:25\"); }} }} mod di {{ struct MailModule {{ sender: {field} }} }}"
            ),
            if field.starts_with("std") { &[] } else { &[1] },
        );
    }
}

#[test]
fn given_di_trait_with_multiple_implementations_when_checked_then_trait_shape_is_legal() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod di { trait MailModule {} struct SmtpModule; struct MemoryModule; impl MailModule for SmtpModule {} impl MailModule for MemoryModule {} }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod di { enum MailModule { Smtp, Memory } }",
        &[1],
    );
}

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
fn given_contract_bound_on_di_module_impl_when_checked_then_the_struct_is_reported() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod api { pub trait Repo {} }\nmod di {\nstruct InvoiceModule<R> { repo: R }\nimpl<R: crate::api::Repo> InvoiceModule<R> { fn new(repo: R) -> Self { Self { repo } } }\nstruct OtherModule<T> { value: T }\nimpl<T: Clone> OtherModule<T> { fn new(value: T) -> Self { Self { value } } }\n}",
        &[3],
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

#[test]
fn given_accessor_constructing_service_in_get_or_init_when_checked_then_only_other_violation_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleShape,
        "mod api { pub trait Repo {} }\nmod file { pub struct FileRepo; impl crate::api::Repo for FileRepo {} impl FileRepo { pub fn new() -> Self { Self } } }\nmod di {\nstruct DeferredModule { repo: std::sync::OnceLock<std::sync::Arc<dyn crate::api::Repo>> }\nimpl DeferredModule { fn new() -> Self { Self { repo: std::sync::OnceLock::new() } } fn repo(&self) -> &std::sync::Arc<dyn crate::api::Repo> { self.repo.get_or_init(|| std::sync::Arc::new(crate::file::FileRepo::new())) } }\nstruct WrongModule { repo: Option<std::sync::Arc<dyn crate::api::Repo>> }\n}",
        &[6],
    );
}

#[test]
fn given_contract_bound_on_di_module_function_when_checked_then_reports_the_struct() {
    for (signature, expected) in [
        (
            "<R: crate::api::InvoiceRepository + 'static>(repository: R) -> Self",
            3,
        ),
        (
            "<R>(repository: R) -> Self where R: crate::api::InvoiceRepository",
            3,
        ),
        (
            "(repository: impl crate::api::InvoiceRepository) -> Self",
            3,
        ),
        (
            "(repository: std::sync::Arc<dyn crate::api::InvoiceRepository>) -> Self",
            0,
        ),
        ("(folder: impl Into<std::path::PathBuf>) -> Self", 0),
    ] {
        let source = format!(
            "mod api {{ pub trait InvoiceRepository {{}} }}\nmod di {{\npub struct InvoiceModule;\nimpl InvoiceModule {{\npub fn new{signature} {{ Self }}\n}}\n}}"
        );
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleShape,
            &source,
            if expected == 0 { &[] } else { &[3] },
        );
    }
}

#[test]
fn given_contract_bound_on_use_case_function_when_checked_then_reports_the_struct() {
    for (signature, expected) in [
        (
            "<R: crate::api::InvoiceRepository>(&self, repository: &R)",
            3,
        ),
        (
            "<R>(&self, repository: &R) where R: crate::api::InvoiceRepository",
            3,
        ),
        (
            "(&self, repository: &impl crate::api::InvoiceRepository)",
            3,
        ),
        ("(&self, repository: &dyn crate::api::InvoiceRepository)", 0),
        ("(&self, label: impl Into<String>)", 0),
    ] {
        let source = format!(
            "mod api {{ pub trait InvoiceRepository {{}} }}\nmod usecase {{\npub struct PayInvoiceUseCase;\nimpl PayInvoiceUseCase {{\npub fn invoke{signature} {{}}\n}}\n}}"
        );
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleShape,
            &source,
            if expected == 0 { &[] } else { &[3] },
        );
    }
}
