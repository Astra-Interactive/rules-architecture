use crate::source::model::CrateSources;
use crate::source::model::SourceTree;
use crate::source::syn::ModuleTreeWalker;
use std::collections::BTreeSet;
use std::fs;

fn walk_root_file(source: &str, has_linkable_target: bool) -> CrateSources {
    let root = tempfile::tempdir().expect("temporary directory");
    fs::create_dir_all(root.path().join("src")).expect("src");
    fs::write(root.path().join("src/lib.rs"), source).expect("root source");
    fs::write(root.path().join("src/included.rs"), "").expect("included source");
    ModuleTreeWalker::new(
        root.path().to_owned(),
        "sample".to_owned(),
        "sample".to_owned(),
    )
    .with_linkable_target(has_linkable_target)
    .with_external_crates(BTreeSet::from([
        "pumpkin_plugin_api".to_owned(),
        "some_dependency".to_owned(),
    ]))
    .walk_crate("src/lib.rs", SourceTree::Src)
}

#[test]
fn given_external_item_macro_with_identifier_arguments_when_walked_without_linkable_target_then_nothing_is_unanalysable()
 {
    for source in [
        "pumpkin_plugin_api::register_plugin!(PumpkinMcPlugin);",
        "some_dependency::declare_entry!();",
        "some_dependency::declare_pair!(First, Second);",
    ] {
        let sources = walk_root_file(source, false);
        assert!(sources.unanalysable.is_empty(), "{source}");
        assert!(!sources.has_linkable_target, "{source}");
    }
}

#[test]
fn given_external_item_macro_when_walked_with_linkable_target_then_the_root_macro_is_unanalysable()
{
    for source in [
        "pumpkin_plugin_api::register_plugin!(PumpkinMcPlugin);",
        "some_dependency::declare_entry!();",
    ] {
        let sources = walk_root_file(source, true);
        assert_eq!(1, sources.unanalysable.len(), "{source}");
        assert!(sources.has_linkable_target, "{source}");
    }
}

#[test]
fn given_local_macro_or_include_or_other_arguments_when_walked_without_linkable_target_then_the_macro_stays_unanalysable()
 {
    for source in [
        "macro_rules! make_items { ($name:ident) => { struct $name; }; }\nmake_items!(Hidden);",
        "crate::make_items!(Hidden);\n#[macro_export] macro_rules! make_items { ($name:ident) => { struct $name; }; }",
        "include!(\"included.rs\");",
        "register_plugin!(PumpkinMcPlugin);",
        "workspace_crate::register_plugin!(PumpkinMcPlugin);",
        "self::make_items!(Hidden);\nmod inner { #[macro_export] macro_rules! make_items { ($name:ident) => { struct $name; }; } }",
        "inner::make_items!(Hidden);\nmod inner { pub use crate::make_items; }\n#[macro_export] macro_rules! make_items { ($name:ident) => { struct $name; }; }",
        "some_dependency::export_entry!(other_crate::Plugin);",
        "some_dependency::export_entry!(Plugin<u8>);",
        "some_dependency::export_entry!(key = value);",
    ] {
        let sources = walk_root_file(source, false);
        assert_eq!(1, sources.unanalysable.len(), "{source}");
    }
}

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
