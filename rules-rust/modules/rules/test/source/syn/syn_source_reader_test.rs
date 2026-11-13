use crate::check::di::CheckModule;
use crate::source::api::SourceReader;
use crate::source::model::SourceTree;
use crate::source::syn::SynSourceReader;
use crate::test::check::usecase::read_case_context;
use std::fs;

#[test]
fn given_shared_physical_source_in_two_targets_when_read_then_one_attribute_is_not_a_duplicate_expectation()
 {
    let context = read_case_context("CrateTargets", "fail-shared-target");
    let crates = SynSourceReader.read_sources(&context.workspace);
    assert_eq!(crates.len(), 2);
    assert_eq!(
        crates
            .iter()
            .map(|sources| sources.expectations.len())
            .sum::<usize>(),
        1
    );
    let report = CheckModule::default()
        .check_workspace_use_case
        .invoke(&context.workspace.root)
        .expect("valid workspace");
    assert!(
        !report
            .violations
            .iter()
            .any(|finding| finding.rule_id.0.contains("Expectation"))
    );
    assert_eq!(report.accepted.len(), 1);
}

#[test]
fn given_two_identical_attributes_on_one_physical_node_when_read_through_two_targets_then_real_duplicate_is_reported()
 {
    let context = read_case_context("CrateTargets", "fail-shared-target");
    let temporary = tempfile::tempdir().expect("temporary workspace");
    for file in &context.workspace.files {
        let source = context.workspace.root.join(file);
        let destination = temporary.path().join(file);
        fs::create_dir_all(destination.parent().expect("file parent"))
            .expect("create fixture directory");
        fs::copy(source, destination).expect("copy source fixture");
    }
    let shared = temporary
        .path()
        .join("modules/sample/src/sample/util/pair_read.rs");
    fs::write(shared, "#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = \"reported as #1\"))] #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = \"reported as #1\"))] pub(crate) fn read_pair() -> (u8, u8) { (1, 2) }\n").expect("write duplicate source");
    let report = CheckModule::default()
        .check_workspace_use_case
        .invoke(temporary.path())
        .expect("valid workspace");
    assert_eq!(
        report
            .violations
            .iter()
            .filter(|finding| finding.rule_id.0 == "MalformedExpectation")
            .count(),
        1
    );
    assert_eq!(report.accepted.len(), 1);
}

#[test]
fn given_published_crate_with_tests_it_when_read_then_tests_it_is_a_source_root_and_an_unpublished_crate_has_none()
 {
    let context = read_case_context("TestMirror", "pass-published-integration");
    let tests_it: Vec<_> = context
        .crates
        .iter()
        .filter(|sources| sources.target == "it")
        .collect();
    assert_eq!(1, tests_it.len());
    let mut paths: Vec<&str> = tests_it[0]
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        [
            "modules/invoice/tests/it/invoice/util/invoice_load_test.rs",
            "modules/invoice/tests/it/main.rs"
        ]
    );
    assert!(
        tests_it[0]
            .modules
            .iter()
            .all(|module| module.tree == SourceTree::TestsIt)
    );
    assert!(
        context
            .crates
            .iter()
            .all(|sources| sources.undeclared_files.is_empty())
    );
    let context = read_case_context("TestMirror", "fail-unpublished-integration");
    assert!(context.crates.iter().all(|sources| sources.target != "it"));
    assert!(
        context
            .crates
            .iter()
            .flat_map(|sources| &sources.files)
            .all(|file| !file.path.contains("tests/it"))
    );
}

#[test]
fn given_tests_it_target_when_read_then_its_root_module_has_no_declaration_and_its_blocks_have_one()
{
    let context = read_case_context("TestMirror", "pass-published-integration");
    let tests_it = context
        .crates
        .iter()
        .find(|sources| sources.target == "it")
        .expect("tests/it target");
    let root = tests_it
        .modules
        .iter()
        .find(|module| module.id.path.is_empty())
        .expect("root module");
    assert!(root.declaration.is_none());
    let block = tests_it
        .modules
        .iter()
        .find(|module| module.id.path == ["invoice"])
        .expect("invoice block");
    assert!(
        block
            .declaration
            .as_ref()
            .is_some_and(|declaration| declaration.is_inline)
    );
}
