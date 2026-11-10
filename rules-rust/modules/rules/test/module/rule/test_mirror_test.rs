use crate::check::api::Rule;
use crate::module::rule::TestMirror;
use crate::source::model::SourceTree;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_findings;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

#[test]
fn given_equivalent_test_cfg_in_production_source_when_checked_then_only_the_cfg_and_test_attributes_report()
 {
    assert_code_findings(
        &TestMirror,
        "TestMirror",
        "mod util {\n    #[cfg(all(test))]\n    pub(crate) fn helper() {}\n    #[cfg(test)]\n    fn plain() {}\n}",
        &[2, 4],
    );
}

#[test]
fn given_tokio_test_function_in_production_source_when_checked_then_the_attribute_reports() {
    assert_code_findings(
        &TestMirror,
        "TestMirror",
        "mod util {\n    #[tokio::test]\n    async fn given_text_when_parsed_then_fails() {}\n    #[test]\n    fn given_text_when_parsed_then_passes() {}\n}",
        &[2, 4],
    );
}

#[test]
fn given_test_file_in_fake_directory_when_checked_then_it_must_end_in_test_like_any_other_test_file()
 {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "#[cfg(test)]\n#[path = \"../test\"]\nmod test {\n    mod invoice {\n        pub(crate) mod fake {\n            pub(crate) use fake_invoice_repository::FakeInvoiceRepository;\n            mod fake_invoice_repository;\n        }\n        mod usecase {\n            mod issue_helper;\n        }\n    }\n}",
        ),
        (
            "test/invoice/fake/fake_invoice_repository.rs",
            "pub(crate) struct FakeInvoiceRepository;\n\n#[test]\nfn given_nothing_when_run_then_passes() {}",
        ),
        (
            "test/invoice/usecase/issue_helper.rs",
            "#[test]\nfn given_nothing_when_run_then_passes() {}",
        ),
    ]);
    assert_placement_findings(
        &TestMirror,
        "TestMirror",
        &context,
        &[("src/lib.rs", 7, "Module"), ("src/lib.rs", 10, "Module")],
    );
}

#[test]
fn given_root_or_test_named_crate_directory_when_checked_then_mirror_paths_are_relative_to_the_member()
 {
    for directory in ["", "modules/test/sample"] {
        let mut context = read_case_context("TestMirror", "pass-mirrored");
        context.workspace.members[0].directory = directory.to_owned();
        let prefix = if directory.is_empty() {
            String::new()
        } else {
            format!("{directory}/")
        };
        for file in &mut context.crates[0].files {
            file.path = format!(
                "{prefix}{}",
                file.path
                    .strip_prefix("modules/sample/")
                    .expect("fixture member")
            );
        }
        assert!(TestMirror.check(&context).is_empty());
        context.workspace.directories.push(format!("{prefix}tests"));
        assert_eq!(TestMirror.check(&context).len(), 1);
    }
}

#[test]
fn given_test_owner_when_checked_then_only_behaviour_can_have_a_mirrored_test() {
    let mut context = read_case_context("TestMirror", "pass-mirrored");
    assert!(TestMirror.check(&context).is_empty());
    context.crates[0].files[0].syntax =
        syn::parse_file("#[cfg(test)] #[path = \"../test\"] mod test;").expect("valid syntax");
    assert!(TestMirror.check(&context).is_empty());
    for syntax in [
        "struct Invoice;",
        "struct Invoice; impl Invoice { const LIMIT: u8 = 1; fn load() {} }",
        "trait Load { fn load() {} }",
    ] {
        context.crates[0].files[1].syntax = syn::parse_file(syntax).expect("valid syntax");
        let found = TestMirror.check(&context);
        assert_eq!(found.len(), usize::from(syntax == "struct Invoice;"));
    }
    context.crates[0].files[0].syntax = syn::parse_file("#[cfg(test)] fn check() {} #[path = \"extra.rs\"] mod extra {} mod test { #[path = \"wrong.rs\"] mod nested {} } fn later() {}").expect("valid syntax");
    assert_eq!(TestMirror.check(&context).len(), 4);
    context
        .workspace
        .directories
        .push("modules/sample/tests".to_owned());
    assert_eq!(TestMirror.check(&context).len(), 5);
}

#[test]
fn given_proc_macro_expansion_tests_when_checked_then_only_it_tree_is_allowed_and_mirrors_production()
 {
    let mut context = read_case_context("TestMirror", "pass-mirrored");
    context.workspace.members[0].targets[0].kinds = vec!["proc-macro".to_owned()];
    context.workspace.directories.extend([
        "modules/sample/tests".to_owned(),
        "modules/sample/tests/it".to_owned(),
    ]);
    context
        .workspace
        .files
        .push("modules/sample/tests/it/main.rs".to_owned());
    assert!(TestMirror.check(&context).is_empty());
    context
        .workspace
        .files
        .push("modules/sample/tests/extra.rs".to_owned());
    assert_eq!(TestMirror.check(&context).len(), 1);
    context.workspace.files.pop();
    context
        .workspace
        .directories
        .push("modules/sample/tests/other".to_owned());
    assert_eq!(TestMirror.check(&context).len(), 1);
    context.workspace.directories.pop();
    let test_file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| file.path.contains("/test/"))
        .expect("test source");
    test_file.path = test_file.path.replace("/test/", "/tests/it/");
    let test_id = test_file.module.clone();
    context.crates[0]
        .modules
        .iter_mut()
        .find(|module| module.id == test_id)
        .expect("test module")
        .tree = SourceTree::TestsIt;
    assert!(TestMirror.check(&context).is_empty());
    let test_file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| file.module == test_id)
        .expect("test source");
    test_file.path = "modules/sample/tests/it/invoice/util/wrong.rs".to_owned();
    assert_eq!(TestMirror.check(&context).len(), 1);
    let test_file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| file.module == test_id)
        .expect("test source");
    test_file.module.path.push("fake".to_owned());
    let module = context.crates[0]
        .modules
        .iter_mut()
        .find(|module| module.id == test_id)
        .expect("test module");
    module.id.path.push("fake".to_owned());
    assert!(TestMirror.check(&context).is_empty());
}
#[test]
fn given_behavior_covered_through_an_entrypoint_when_checked_then_no_per_type_test_file_is_required()
 {
    let context = crate::test::declaration::util::read_code_context(
        "mod service { struct Worker; impl Worker { fn run(&self) {} } struct Owner { worker: Worker } impl Owner { fn run(&self) { self.worker.run(); } } }",
    );
    assert!(TestMirror.check(&context).is_empty());
    let context = read_case_context("TestMirror", "pass-mirrored");
    assert!(TestMirror.check(&context).is_empty());
}
