use crate::check::api::Rule;
use crate::module::rule::UseCaseShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_aliased_usecases_when_stored_or_invoked_then_dependencies_are_reported_and_other_invokes_pass()
 {
    let context = read_shape_context(
        "mod usecase { #[cfg(unix)] struct ReadInvoiceUseCase; #[cfg(unix)] impl ReadInvoiceUseCase { pub fn invoke(&self) {} } use ReadInvoiceUseCase as Other; struct ComputeInvoiceUseCase { reader: Other } impl ComputeInvoiceUseCase { pub fn invoke(&self) {} } struct PayInvoiceUseCase; impl PayInvoiceUseCase { pub fn invoke(&self) { let reader: Other = Other; reader.invoke(); } } struct ShowInvoiceUseCase; struct Renderer; impl Renderer { fn invoke(&self) {} } impl ShowInvoiceUseCase { pub fn invoke(&self) { let reader = Renderer; reader.invoke(); } } }",
        &[],
    );
    let mut subjects: Vec<String> = UseCaseShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "case_sample::usecase::ComputeInvoiceUseCase",
            "case_sample::usecase::PayInvoiceUseCase",
            "case_sample::usecase::Renderer"
        ]
    );
}

#[test]
fn given_usecase_methods_when_checked_then_missing_await_and_extra_api_are_reported() {
    let context = read_shape_context(
        "mod usecase { struct ComputeInvoiceUseCase; impl ComputeInvoiceUseCase { pub async fn invoke(&self) {} } struct PayInvoiceUseCase; impl PayInvoiceUseCase { pub fn invoke(&self) {} pub fn refresh(&self) {} } struct ReadInvoiceUseCase; impl ReadInvoiceUseCase { pub async fn invoke(&self) { std::future::ready(()).await; } } }",
        &[],
    );
    let mut subjects: Vec<String> = UseCaseShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "case_sample::usecase::ComputeInvoiceUseCase",
            "case_sample::usecase::PayInvoiceUseCase"
        ]
    );
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("UseCaseShape", "fail-declaration-shape");
    let mut actual: Vec<String> = UseCaseShape
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "modules/sample/src/usecase/calculate_use_case.rs:1 case_sample::usecase::CalculateUseCase"
        ]
    );
    let context = read_case_context("UseCaseShape", "pass-declaration-shape");
    assert!(UseCaseShape.check(&context).is_empty());
}

#[test]
fn given_concrete_and_generic_use_case_fields_when_checked_then_trait_object_and_business_logic_pass()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::UseCaseShape,
        "mod api { pub trait Repo {} } mod file { pub struct Repo; impl crate::api::Repo for Repo {} } mod usecase {\nstruct LoadInvoiceUseCase { repo: crate::file::Repo } impl LoadInvoiceUseCase { fn invoke(&self) {} }\nstruct SaveInvoiceUseCase<T> { repo: T } impl<T> SaveInvoiceUseCase<T> { fn invoke(&self) {} } }",
        &[2, 3],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::UseCaseShape,
        "mod usecase { struct LoadInvoiceUseCase { repo: std::sync::Arc<dyn Send + Sync> } impl LoadInvoiceUseCase { fn invoke(&self) { let result = 1; } } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::UseCaseShape,
        "mod usecase { struct LoadInvoiceUseCase { repo: String } impl LoadInvoiceUseCase { fn invoke(&self) -> usize { self.repo.len() } } }",
        &[1],
    );
}

#[test]
fn given_di_alias_shape_when_checked_then_preserves_defined_exceptions() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::UseCaseShape,
        "mod api { pub trait Repo {} } mod file { pub struct Repo; #[cfg(unix)] impl crate::api::Repo for Repo {} } mod usecase { #[cfg(not(unix))] struct LoadInvoiceUseCase { repo: crate::file::Repo } #[cfg(not(unix))] impl LoadInvoiceUseCase { fn invoke(&self) {} } }",
        &[],
    );
}
