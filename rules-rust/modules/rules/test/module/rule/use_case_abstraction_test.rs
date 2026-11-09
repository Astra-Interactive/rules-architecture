use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::UseCaseAbstraction;
use crate::test::module::util::read_shape_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context =
        crate::test::check::usecase::read_case_context("UseCaseAbstraction", "fail-phase3-role");
    let mut actual: Vec<String> = UseCaseAbstraction
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
        crate::test::check::usecase::read_case_context("UseCaseAbstraction", "pass-phase3-role");
    assert!(UseCaseAbstraction.check(&context).is_empty());
}

#[test]
fn given_usecase_abstraction_when_checked_then_only_app_api_usecase_is_accepted() {
    let mut context = read_shape_context("mod usecase { trait ComputeInvoiceUseCase {} }", &[]);
    let findings = UseCaseAbstraction.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.subject,
        "case_sample::usecase::ComputeInvoiceUseCase"
    );
    context.project_config.crates[0].role = Some(Role::Api);
    assert!(UseCaseAbstraction.check(&context).is_empty());
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nproduct-kind='platform-plugin'",
    )
    .expect("manifest");
    assert_eq!(UseCaseAbstraction.check(&context).len(), 1);
}
