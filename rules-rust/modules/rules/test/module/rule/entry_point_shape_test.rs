use crate::check::api::Rule;
use crate::module::rule::EntryPointShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_cdylib_host_items_when_checked_then_only_hidden_configured_host_reports() {
    let mut context = read_shape_context("fn start_host() {} pub fn shutdown_host() {}", &[]);
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    context.project_config.crates[0].host_items =
        vec!["start_host".to_owned(), "shutdown_host".to_owned()];
    context.workspace.members[0].targets[0].kinds = vec!["cdylib".to_owned()];
    let findings = EntryPointShape.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case_sample::start_host");
    context.project_config.crates[0].host_items.clear();
    assert!(EntryPointShape.check(&context).is_empty());
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("EntryPointShape", "fail-declaration-shape");
    let mut actual: Vec<String> = EntryPointShape
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
            "instances/runner/src/main.rs:1 case_runner::model",
            "instances/runner/src/main.rs:2 case_runner::main"
        ]
    );
    let context = read_case_context("EntryPointShape", "pass-declaration-shape");
    assert!(EntryPointShape.check(&context).is_empty());
}
