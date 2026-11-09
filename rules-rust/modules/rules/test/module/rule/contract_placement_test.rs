use crate::check::api::Rule;
use crate::module::rule::ContractPlacement;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_reexported_workspace_usecase_when_implemented_then_only_own_sibling_impl_usecase_is_accepted()
 {
    let mut context = crate::test::dependency::util::read_role_context(
        crate::config::model::Role::Impl,
        crate::config::model::Role::Api,
        crate::workspace::model::DependencyKind::Normal,
    );
    let mut consumer = read_shape_context("mod usecase { struct DefaultComputeInvoiceUseCase; #[cfg(unix)] impl provider_alias::usecase::Alias for DefaultComputeInvoiceUseCase {} }", &[]).indexes.remove(0);
    let mut provider = read_shape_context("mod usecase { #[cfg(unix)] pub trait ComputeInvoiceUseCase {} pub use ComputeInvoiceUseCase as Alias; }", &[]).indexes.remove(0);
    consumer.crate_name = "case-consumer".to_owned();
    consumer.dependencies = vec!["provider_alias".to_owned()];
    provider.crate_name = "case-provider".to_owned();
    context.workspace.members[0].dependencies[0].code_name = "provider_alias".to_owned();
    context.indexes = vec![consumer, provider];
    assert!(ContractPlacement.check(&context).is_empty());
    context.workspace.members[0].directory = "modules/other/impl".to_owned();
    let findings = ContractPlacement.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.subject,
        "case_consumer::usecase::DefaultComputeInvoiceUseCase"
    );
}

#[test]
fn given_default_usecase_in_app_api_when_checked_then_implementation_requires_sibling_impl_unit() {
    let mut context = read_shape_context(
        "mod usecase { trait ComputeInvoiceUseCase {} struct DefaultComputeInvoiceUseCase; impl ComputeInvoiceUseCase for DefaultComputeInvoiceUseCase {} }",
        &[],
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Api);
    let findings = ContractPlacement.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.subject,
        "case_sample::usecase::DefaultComputeInvoiceUseCase"
    );
}

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context =
        crate::test::check::usecase::read_case_context("ContractPlacement", "fail-phase3-role");
    let mut actual: Vec<String> = ContractPlacement
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.unwrap_or(0),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "modules/sample/src/usecase/compute_invoice_use_case.rs:1 case_sample::usecase::ComputeInvoiceUseCase"
        ]
    );
    let context =
        crate::test::check::usecase::read_case_context("ContractPlacement", "pass-phase3-role");
    assert!(ContractPlacement.check(&context).is_empty());
}

#[test]
fn given_documented_contract_exceptions_when_checked_then_only_unqualified_and_domain_parameter_traits_report()
 {
    let mut context = read_shape_context(
        "mod model { struct Value; trait ValueFactory {} trait StrayFactory {} } mod util { trait ExternalExt {} } mod usecase { trait ComputeInvoiceUseCase {} } mod command { trait State {} struct DefaultState; impl State for DefaultState {} trait Registration { fn register(&self, path: &std::path::Path); } trait DomainRegistration { fn register(&self, value: crate::model::Value); } }",
        &[],
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Api);
    let mut subjects: Vec<String> = ContractPlacement
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "case_sample::command::DomainRegistration",
            "case_sample::model::StrayFactory"
        ]
    );
}

#[test]
fn given_reexported_trait_and_cfg_impl_when_checked_then_api_and_stub_use_defining_trait_identity()
{
    let context = read_shape_context(
        "mod api { #[cfg(unix)] trait Contract {} pub use Contract as Alias; struct Value; #[cfg(unix)] impl Alias for Value {} } mod stub { struct Stub; #[cfg(unix)] impl crate::api::Alias for Stub {} fn unrelated() {} }",
        &[],
    );
    let findings = ContractPlacement.check(&context);
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.subject == "case_sample::stub::unrelated")
    );
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("ContractPlacement", "fail-declaration-shape");
    let mut actual: Vec<String> = ContractPlacement
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
        ["modules/sample/src/model/invoice_repository.rs:1 case_sample::model::InvoiceRepository"]
    );
    let context = read_case_context("ContractPlacement", "pass-declaration-shape");
    assert!(ContractPlacement.check(&context).is_empty());
}
