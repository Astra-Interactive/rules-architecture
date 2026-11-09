use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::CoreShape;
use crate::test::module::util::read_shape_context;

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context = crate::test::check::usecase::read_case_context("CoreShape", "fail-phase3-role");
    let mut actual: Vec<String> = CoreShape
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
            "modules/sample/src/api/invoice_repository.rs:1 case_sample::api::InvoiceRepository",
            "modules/sample/src/lib.rs:1 case_sample::usecase"
        ]
    );
    let context = crate::test::check::usecase::read_case_context("CoreShape", "pass-phase3-role");
    assert!(CoreShape.check(&context).is_empty());
}

#[test]
fn given_core_roles_when_checked_then_repository_and_usecase_kind_are_reported_in_each_role() {
    let mut context =
        read_shape_context("mod api { trait InvoiceRepository {} } mod usecase {}", &[]);
    assert!(CoreShape.check(&context).is_empty());
    for role in [Role::Core, Role::CoreApi, Role::CorePlatform, Role::CoreUi] {
        context.project_config.crates[0].role = Some(role);
        let mut subjects: Vec<String> = CoreShape
            .check(&context)
            .iter()
            .map(|finding| finding.finding.subject.clone())
            .collect();
        subjects.sort();
        assert_eq!(
            subjects,
            [
                "case_sample::api::InvoiceRepository",
                "case_sample::usecase"
            ]
        );
    }
}
