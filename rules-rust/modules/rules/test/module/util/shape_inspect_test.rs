use crate::check::model::RuleContext;
use crate::source::api::SourceReader;
use crate::source::syn::{CrateIndexBuilder, SynSourceReader};
use crate::test::check::usecase::read_case_context;
use std::fs;

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
