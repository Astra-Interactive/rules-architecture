use crate::source::model::SourceTree;
use crate::source::syn::ModuleTreeWalker;
use std::fs;

#[test]
fn given_missing_root_source_when_walked_then_io_failure_is_reported_at_the_crate_boundary() {
    let root = tempfile::tempdir().expect("temporary directory");
    let sources = ModuleTreeWalker::new(
        root.path().to_owned(),
        "sample".to_owned(),
        "sample".to_owned(),
    )
    .walk_crate("missing.rs", SourceTree::Src);
    assert_eq!(sources.unanalysable.len(), 1);
    assert!(sources.unanalysable[0].message.contains("missing.rs"));
    assert!(sources.files.is_empty());
}

#[test]
fn given_conditional_declarations_for_one_module_when_walked_then_physical_expectations_are_loaded_once()
 {
    let root = tempfile::tempdir().expect("temporary directory");
    fs::create_dir_all(root.path().join("src")).expect("src");
    fs::write(
        root.path().join("src/lib.rs"),
        "#[cfg(unix)] mod shared; #[cfg(windows)] mod shared;",
    )
    .expect("root source");
    fs::write(root.path().join("src/shared.rs"), "#![cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"module\"))]\n#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"node\"))] fn example() {}").expect("shared source");
    let sources = ModuleTreeWalker::new(
        root.path().to_owned(),
        "sample".to_owned(),
        "sample".to_owned(),
    )
    .walk_crate("src/lib.rs", SourceTree::Src);
    assert!(sources.unanalysable.is_empty());
    assert_eq!(sources.files.len(), 2);
    assert_eq!(sources.expectations.len(), 2);
}

#[test]
fn given_two_module_paths_for_one_file_when_walked_then_module_scopes_remain_and_node_scopes_are_unique()
 {
    let root = tempfile::tempdir().expect("temporary directory");
    fs::create_dir_all(root.path().join("src")).expect("src");
    fs::write(
        root.path().join("src/lib.rs"),
        "#[path=\"shared.rs\"] mod test; #[path=\"shared.rs\"] mod fixture;",
    )
    .expect("root source");
    fs::write(root.path().join("src/shared.rs"), "#![cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"module\"))]\n#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"first\"))]\n#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"second\"))] fn example() {}").expect("shared source");
    let sources = ModuleTreeWalker::new(
        root.path().to_owned(),
        "sample".to_owned(),
        "sample".to_owned(),
    )
    .walk_crate("src/lib.rs", SourceTree::Src);
    assert!(sources.unanalysable.is_empty());
    assert_eq!(sources.files.len(), 3);
    assert_eq!(sources.expectations.len(), 4);
}

#[test]
fn given_external_test_root_with_path_when_walked_then_the_target_file_is_loaded() {
    let root = tempfile::tempdir().expect("temporary directory");
    fs::create_dir_all(root.path().join("src")).expect("src");
    fs::create_dir_all(root.path().join("test")).expect("test");
    fs::write(
        root.path().join("src/lib.rs"),
        "#[path=\"../test/case.rs\"] mod test;",
    )
    .expect("source");
    fs::write(root.path().join("test/case.rs"), "fn example() {}").expect("test source");
    let sources = ModuleTreeWalker::new(
        root.path().to_owned(),
        "sample".to_owned(),
        "sample".to_owned(),
    )
    .walk_crate("src/lib.rs", SourceTree::Src);
    assert!(sources.unanalysable.is_empty());
    assert!(sources.files.iter().any(|file| file.path == "test/case.rs"));
}
