use crate::check::model::RuleContext;
use crate::source::api::SourceReader;
use crate::source::syn::{CrateIndexBuilder, SynSourceReader};
use crate::test::check::usecase::read_case_context;
use std::fs;

pub(crate) fn read_workspace_shape_context(members: &[(&str, &str, &str)]) -> RuleContext {
    use crate::workspace::api::WorkspaceReader;
    let mut context = read_case_context("RootFileShape", "pass-tree");
    let directory = tempfile::tempdir().expect("temporary workspace");
    let names: Vec<_> = members
        .iter()
        .map(|(name, _, _)| format!("{name:?}"))
        .collect();
    fs::write(directory.path().join("Cargo.toml"), format!("[workspace]\nresolver=\"3\"\nmembers=[{}]\n[workspace.metadata.klibs-rules]\nproduct-kind=\"app\"\n", names.join(","))).expect("workspace manifest");
    for (name, dependencies, source) in members {
        let root = directory.path().join(name);
        fs::create_dir_all(root.join("src")).expect("member source directory");
        fs::write(
            root.join("Cargo.toml"),
            format!("[package]\nname={name:?}\nedition=\"2024\"\n[dependencies]\n{dependencies}"),
        )
        .expect("member manifest");
        fs::write(root.join("src/lib.rs"), source).expect("member source");
    }
    context.workspace = crate::workspace::cargo::CargoWorkspaceReader::new(
        crate::workspace::util::resolve_cargo_path(),
    )
    .read_workspace(directory.path())
    .expect("read workspace");
    context.project_config = crate::config::util::parse_project_config(&context.workspace)
        .expect("project configuration");
    context.crates = SynSourceReader.read_sources(&context.workspace);
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    context
}

pub(crate) fn read_shape_context(source: &str, files: &[(&str, &str)]) -> RuleContext {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    let directory = tempfile::tempdir().expect("temporary source tree");
    let source_root = directory.path().join("modules/sample/src");
    fs::create_dir_all(&source_root).expect("source root");
    fs::write(source_root.join("lib.rs"), source).expect("root source");
    for (path, text) in files {
        let path = source_root.join(path);
        fs::create_dir_all(path.parent().expect("source parent")).expect("parent");
        fs::write(path, text).expect("file module");
    }
    context.workspace.root = directory.path().to_owned();
    context.crates = SynSourceReader.read_sources(&context.workspace);
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    context
}
