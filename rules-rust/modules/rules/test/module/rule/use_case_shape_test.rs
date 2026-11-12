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
        "mod usecase { struct LoadInvoiceUseCase { repo: std::sync::Arc<dyn Send + Sync> } impl LoadInvoiceUseCase { pub fn invoke(&self) { let result = 1; } } }",
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
        "mod api { pub trait Repo {} } mod file { pub struct Repo; #[cfg(unix)] impl crate::api::Repo for Repo {} } mod usecase { #[cfg(not(unix))] struct LoadInvoiceUseCase { repo: crate::file::Repo } #[cfg(not(unix))] impl LoadInvoiceUseCase { pub fn invoke(&self) {} } }",
        &[],
    );
}

#[test]
fn given_tdd_use_case_shape_ufcs_forwarding_when_linted_then_reports_the_use_case_node() {
    for forwarding in [
        "Counter::read(&*self.counter)",
        "<dyn Counter as Counter>::read(&*self.counter)",
    ] {
        crate::test::module::util::assert_di_lines(
            &UseCaseShape,
            &format!(
                "mod api {{ pub trait Counter {{ fn read(&self) -> u32; }} }}\nmod usecase {{ use crate::api::Counter; use std::sync::Arc; struct ReadCounterUseCase {{ counter: Arc<dyn crate::api::Counter> }} impl ReadCounterUseCase {{ pub fn new(counter: Arc<dyn crate::api::Counter>) -> Self {{ Self {{ counter }} }} pub fn invoke(&self) -> u32 {{ {forwarding} }} }} }}"
            ),
            &[2],
        );
    }
}
#[test]
fn given_forwarded_bindings_and_private_invoke_when_checked_then_only_business_decision_with_public_entry_passes()
 {
    for body in [
        "let result = self.repo.read(); result",
        "let result = self.repo.read(); return result;",
    ] {
        crate::test::module::util::assert_di_lines(
            &UseCaseShape,
            &format!(
                "mod usecase {{ struct ReadInvoiceUseCase {{ repo: String }} impl ReadInvoiceUseCase {{ pub fn invoke(&self) {{ {body} }} }} }}"
            ),
            &[1],
        );
    }
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod usecase { struct ReadInvoiceUseCase; impl ReadInvoiceUseCase { fn invoke(&self) {} } }",
        &[1],
    );
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod usecase { struct ReadInvoiceUseCase { repo: String } impl ReadInvoiceUseCase { pub(crate) fn invoke(&self) -> bool { let result = self.repo.is_empty(); !result } } }",
        &[],
    );
}

#[test]
fn given_usecase_contract_constructor_arguments_when_checked_then_only_arc_trait_objects_pass() {
    for parameter in [
        "Box<dyn crate::api::Repo>",
        "&dyn crate::api::Repo",
        "std::sync::Arc<dyn crate::api::Repo>",
    ] {
        crate::test::module::util::assert_di_lines(
            &UseCaseShape,
            &format!(
                "mod api {{ pub trait Repo {{}} }} mod usecase {{ struct ReadInvoiceUseCase; impl ReadInvoiceUseCase {{ pub fn new(repo: {parameter}) -> Self {{ Self }} pub fn invoke(&self) {{}} }} }}"
            ),
            if parameter.starts_with("std::sync::Arc") {
                &[]
            } else {
                &[1]
            },
        );
    }
}
#[test]
fn given_read_modify_write_or_adapted_argument_when_checked_then_business_orchestration_is_preserved()
 {
    for body in [
        "let balance = self.repo.read_balance(); self.repo.write_balance(balance + 1);",
        "self.repo.write_balance(balance + 1)",
        "let first = self.repo.read(); let second = self.repo.read(); second",
    ] {
        crate::test::module::util::assert_di_lines(
            &UseCaseShape,
            &format!(
                "mod usecase {{ struct PayInvoiceUseCase {{ repo: String }} impl PayInvoiceUseCase {{ pub fn invoke(&self, balance: i64) {{ {body} }} }} }}"
            ),
            &[],
        );
    }
}
#[test]
fn given_trait_invoke_and_aliased_shared_contract_when_checked_then_the_public_entry_and_arc_identity_are_preserved()
 {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod api { use std::sync::Arc; pub trait Repo {} pub type SharedRepo = Arc<dyn Repo>; } trait Invocation { fn invoke(&self); } mod usecase { struct ReadInvoiceUseCase; impl ReadInvoiceUseCase { pub fn new(repo: crate::api::SharedRepo) -> Self { Self } } impl crate::Invocation for ReadInvoiceUseCase { fn invoke(&self) {} } }",
        &[],
    );
}

#[test]
fn given_typed_local_forwarding_the_contract_result_when_checked_then_the_pass_through_use_case_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod usecase {\nstruct ReadCounterUseCase { counter: String } impl ReadCounterUseCase { pub fn invoke(&self) -> u32 { let result: u32 = self.counter.read(); result } }\nstruct CheckCounterUseCase { counter: String } impl CheckCounterUseCase { pub fn invoke(&self) -> bool { let result: bool = self.counter.is_empty(); !result } } }",
        &[2],
    );
}

#[test]
fn given_try_and_wrapped_forwarding_of_the_contract_result_when_checked_then_the_pass_through_use_case_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod usecase {\nstruct LoadRatingUseCase { repository: String } impl LoadRatingUseCase { pub fn invoke(&self, id: u32) -> Result<u32, String> { let rating = self.repository.load(id)?; Ok(rating) } }\nstruct ReadRatingUseCase { ratings: String } impl ReadRatingUseCase { pub fn invoke(&self, id: u32) -> Option<u32> { Some(self.ratings.rating(id)) } }\nstruct ScoreRatingUseCase { repository: String } impl ScoreRatingUseCase { pub fn invoke(&self, id: u32) -> Result<u32, String> { let rating = self.repository.load(id)?; Ok(rating.score) } } }",
        &[2, 3],
    );
}

#[test]
fn given_block_body_forwarding_the_contract_call_when_checked_then_the_pass_through_use_case_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod usecase {\nstruct ReadCounterUseCase { counter: String } impl ReadCounterUseCase { pub fn invoke(&self) -> u32 { { self.counter.read() } } }\nstruct CheckCounterUseCase { counter: String } impl CheckCounterUseCase { pub fn invoke(&self) -> bool { { !self.counter.is_empty() } } } }",
        &[2],
    );
}

#[test]
fn given_generic_constructor_parameter_bounded_by_a_contract_when_checked_then_only_arc_trait_objects_pass()
 {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod api { pub trait Repo {} }\nmod usecase {\nstruct ReadInvoiceUseCase; impl ReadInvoiceUseCase { pub fn new<R: crate::api::Repo>(repo: R) -> Self { Self } pub fn invoke(&self) {} }\nstruct PayInvoiceUseCase; impl PayInvoiceUseCase { pub fn new<R>(repo: R) -> Self where R: crate::api::Repo { Self } pub fn invoke(&self) {} }\nstruct ShowInvoiceUseCase; impl ShowInvoiceUseCase { pub fn new(repo: std::sync::Arc<dyn crate::api::Repo>) -> Self { Self } pub fn invoke(&self) {} } }",
        &[3, 4],
    );
}

#[test]
fn given_cfg_disabled_invoke_when_checked_then_the_use_case_without_an_active_entry_is_reported() {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod api { pub trait Repo {} pub trait Clock {} }\nmod usecase {\nstruct ReadInvoiceUseCase { repo: std::sync::Arc<dyn crate::api::Repo>, clock: std::sync::Arc<dyn crate::api::Clock> } impl ReadInvoiceUseCase { pub fn new(repo: std::sync::Arc<dyn crate::api::Repo>, clock: std::sync::Arc<dyn crate::api::Clock>) -> Self { Self { repo, clock } } #[cfg(any())] pub(crate) fn invoke(&self) {} }\nstruct PayInvoiceUseCase; impl PayInvoiceUseCase { #[cfg(any())] fn invoke(&self) {} pub fn invoke(&self) {} } }",
        &[3],
    );
}

#[test]
fn given_use_case_with_contract_fields_and_no_constructor_when_checked_then_it_is_reported() {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod api { pub trait Repo {} pub trait Clock {} }\nmod usecase {\nstruct PayInvoiceUseCase { repo: std::sync::Arc<dyn crate::api::Repo>, clock: std::sync::Arc<dyn crate::api::Clock> } impl PayInvoiceUseCase { pub fn invoke(&self) {} }\nstruct ReadInvoiceUseCase { guard: std::sync::Arc<dyn Send + Sync> } impl ReadInvoiceUseCase { pub fn invoke(&self) {} }\nstruct ShowInvoiceUseCase { repo: std::sync::Arc<dyn crate::api::Repo> } impl ShowInvoiceUseCase { pub fn new(repo: std::sync::Arc<dyn crate::api::Repo>) -> Self { Self { repo } } pub fn invoke(&self) {} } }",
        &[3],
    );
}

#[test]
fn given_free_function_in_usecase_kind_when_checked_then_the_function_is_reported() {
    crate::test::module::util::assert_di_lines(
        &UseCaseShape,
        "mod api { pub trait Repo { fn save(&self); } }\nmod usecase {\npub(crate) fn issue_invoice(repository: &dyn crate::api::Repo) { repository.save(); }\nstruct PayInvoiceUseCase; impl PayInvoiceUseCase { pub fn invoke(&self) {} } }\nmod util { pub(crate) fn double(value: u32) -> u32 { value * 2 } }",
        &[3],
    );
}

#[test]
fn given_test_and_helper_functions_in_test_tree_usecase_kind_when_checked_then_only_the_src_function_is_reported()
 {
    let context = crate::test::module::util::read_shape_context(
        "mod usecase {\npub(crate) fn issue_invoice() {}\n}\n#[cfg(test)] #[path = \"../test\"] mod test { mod order { mod usecase { mod create_order_use_case_test; } } }",
        &[(
            "../test/order/usecase/create_order_use_case_test.rs",
            "#[test]\nfn given_order_when_created_then_it_is_saved() { build_order(); }\nfn build_order() {}\n",
        )],
    );
    let locations: Vec<String> = UseCaseShape
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line")
            )
        })
        .collect();
    assert_eq!(locations, ["modules/sample/src/lib.rs:2"]);
}

fn read_use_case_subjects(context: &crate::check::model::RuleContext) -> Vec<String> {
    UseCaseShape
        .check(context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect()
}

#[test]
fn given_use_case_of_another_crate_invoked_inside_closure_or_async_block_when_checked_then_the_caller_is_reported()
 {
    for call in [
        "let _all: Vec<u32> = ids.iter().map(|id| load.invoke(*id)).collect();",
        "let _task = async { load.invoke(1) };",
    ] {
        let source = format!(
            "mod usecase {{ struct ReadInvoicesUseCase; impl ReadInvoicesUseCase {{ pub fn invoke(&self, ids: &[u32], load: &provider::usecase::LoadInvoiceUseCase) {{ {call} }} }} struct ShowInvoicesUseCase; impl ShowInvoicesUseCase {{ pub fn invoke(&self, ids: &[u32], renderer: &crate::render::Renderer) {{ let _all: Vec<u32> = ids.iter().map(|id| renderer.invoke(*id)).collect(); }} }} }} mod render {{ pub struct Renderer; impl Renderer {{ pub fn invoke(&self, id: u32) -> u32 {{ id }} }} }}"
        );
        let context = crate::test::module::util::read_workspace_shape_context(&[
            (
                "provider",
                "",
                "pub mod usecase { pub struct LoadInvoiceUseCase; impl LoadInvoiceUseCase { pub fn invoke(&self, id: u32) -> u32 { id } } }",
            ),
            ("consumer", "provider={path=\"../provider\"}", &source),
        ]);
        let subjects = read_use_case_subjects(&context);
        assert_eq!(subjects.len(), 1, "{subjects:?}");
        assert!(
            subjects[0].ends_with("::usecase::ReadInvoicesUseCase"),
            "{subjects:?}"
        );
    }
}

#[test]
fn given_use_case_of_another_crate_awaited_when_checked_then_the_caller_is_reported() {
    let context = crate::test::module::util::read_workspace_shape_context(&[
        (
            "provider",
            "",
            "pub mod usecase { pub struct LoadInvoiceUseCase; impl LoadInvoiceUseCase { pub async fn invoke(&self, id: u32) -> u32 { std::future::ready(id).await } } }",
        ),
        (
            "consumer",
            "provider={path=\"../provider\"}",
            "mod usecase { struct ReadInvoicesUseCase; impl ReadInvoicesUseCase { pub async fn invoke(&self, load: &provider::usecase::LoadInvoiceUseCase) -> u32 { load.invoke(1).await } } struct ShowInvoicesUseCase; impl ShowInvoicesUseCase { pub async fn invoke(&self) { std::future::ready(()).await; } } }",
        ),
    ]);
    let subjects = read_use_case_subjects(&context);
    assert_eq!(subjects.len(), 1, "{subjects:?}");
    assert!(
        subjects[0].ends_with("::usecase::ReadInvoicesUseCase"),
        "{subjects:?}"
    );
}
