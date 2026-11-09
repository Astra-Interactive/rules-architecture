use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::ContractUnitShape;
use crate::test::module::util::read_shape_context;

#[test]
fn given_phase3_role_fixtures_when_checked_then_exact_wrong_sites_report_and_correct_roles_pass() {
    let context =
        crate::test::check::usecase::read_case_context("ContractUnitShape", "fail-phase3-role");
    let mut actual: Vec<String> = ContractUnitShape
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
            "modules/sample/src/model/invoice.rs:1 case_sample::model::Invoice",
            "modules/sample/src/model/invoice.rs:2 case_sample::model::compute_value"
        ]
    );
    let context =
        crate::test::check::usecase::read_case_context("ContractUnitShape", "pass-phase3-role");
    assert!(ContractUnitShape.check(&context).is_empty());
}

#[test]
fn given_wire_types_and_function_bodies_when_checked_then_derives_and_data_only_contract_are_enforced()
 {
    let mut context = read_shape_context(
        "mod model { #[derive(serde::Serialize, serde::Deserialize)] struct Invoice; struct MissingWire; const ROUTE: &str = \"invoice\"; fn compute_value() {} trait Contract { fn invoke(&self) {} } }",
        &[],
    );
    assert!(ContractUnitShape.check(&context).is_empty());
    context.project_config.crates[0].role = Some(Role::Contract);
    let mut subjects: Vec<String> = ContractUnitShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        [
            "case_sample::model::Contract",
            "case_sample::model::MissingWire",
            "case_sample::model::compute_value"
        ]
    );
    let mut context = read_shape_context(
        "mod model { #[derive(serde::Serialize, serde::Deserialize)] struct Invoice; const ROUTE: &str = \"invoice\"; }",
        &[],
    );
    context.project_config.crates[0].role = Some(Role::Contract);
    assert!(ContractUnitShape.check(&context).is_empty());
}
