use crate::check::api::Rule;
use crate::module::rule::RootModulePlacement;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_cfg_roots_and_configured_name_when_checked_then_exclusive_occurrences_pass_and_duplicates_report()
 {
    let mut context = read_shape_context(
        "mod di { mod root_module; }",
        &[(
            "di/root_module.rs",
            "#[cfg(unix)] struct RootModule; #[cfg(not(unix))] struct RootModule;",
        )],
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    assert!(RootModulePlacement.check(&context).is_empty());
    context.indexes[0].items[1].cfg = context.indexes[0].items[0].cfg.clone();
    assert_eq!(RootModulePlacement.check(&context).len(), 2);
    for item in &mut context.indexes[0].items {
        item.name = "CompositionRoot".to_owned();
    }
    assert!(RootModulePlacement.check(&context).is_empty());
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nroot-module='CompositionRoot'",
    )
    .expect("manifest");
    assert_eq!(RootModulePlacement.check(&context).len(), 2);
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("RootModulePlacement", "fail-declaration-shape");
    let mut actual: Vec<String> = RootModulePlacement
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
        ["instances/runner/src/model/root_module.rs:1 case_runner::model::RootModule"]
    );
    let context = read_case_context("RootModulePlacement", "pass-declaration-shape");
    assert!(RootModulePlacement.check(&context).is_empty());
}
