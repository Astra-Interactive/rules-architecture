#[test]
fn given_entry_main_when_checked_then_runtime_root_enable_and_synchronous_disable_are_required() {
    let source = "mod di { pub struct RootModule; impl RootModule { pub fn new() -> Self { Self } } }\n fn main() { let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async { let root = di::RootModule::new(); root.on_enable(); root }); root.on_disable(); }";
    let mut context = crate::test::declaration::util::read_code_context(source);
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    assert!(crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).is_empty());
    let mut context = crate::test::declaration::util::read_code_context("fn main() {} ");
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    let findings = crate::check::api::Rule::check(&crate::module::rule::MainShape, &context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.location.line, Some(1));
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::MainShape,
            "MainShape",
            half,
        );
    }
}
