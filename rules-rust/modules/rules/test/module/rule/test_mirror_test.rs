use crate::check::api::Rule;
use crate::module::rule::TestMirror;
use crate::source::model::SourceTree;
use crate::test::check::usecase::read_case_context;

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
