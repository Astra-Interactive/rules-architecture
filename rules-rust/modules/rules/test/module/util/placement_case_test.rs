use crate::check::api::Rule;
use crate::check::model::{Anchor, RuleContext};
use crate::source::model::SourceTree;
use crate::source::syn::{CrateIndexBuilder, ModuleTreeWalker};
use crate::test::declaration::util::read_code_context;
use std::fs;
use std::path::Path;

pub(crate) fn read_placement_context(files: &[(&str, &str)]) -> RuleContext {
    let mut context = read_code_context("");
    let directory = tempfile::tempdir().expect("temporary placement sources");
    for (path, source) in files {
        let path = directory.path().join(path);
        fs::create_dir_all(path.parent().expect("file parent")).expect("source directories");
        fs::write(path, source).expect("source fixture");
    }
    let sources = ModuleTreeWalker::new(
        directory.path().to_owned(),
        context.crates[0].crate_name.clone(),
        context.crates[0].target.clone(),
    )
    .walk_crate("src/lib.rs", SourceTree::Src);
    let dependencies = context.indexes[0].dependencies.clone();
    context.indexes = vec![CrateIndexBuilder::new(&sources, dependencies).build()];
    context.crates = vec![sources];
    context
}

pub(crate) fn assert_placement_findings(
    rule: &dyn Rule,
    name: &str,
    context: &RuleContext,
    expected: &[(&str, u32, &str)],
) {
    for index in &context.indexes {
        assert!(
            index.unanalysable.is_empty(),
            "unexpected unresolved source: {:?}",
            index.unanalysable
        );
    }
    for sources in &context.crates {
        assert!(
            sources.unanalysable.is_empty(),
            "source fixture must parse and load: {:?}",
            sources.unanalysable
        );
    }
    let findings = rule.check(context);
    let mut actual = findings
        .iter()
        .map(|finding| {
            assert_eq!(name, finding.finding.rule_id.0);
            let anchor = match &finding.anchor {
                Anchor::Node { .. } => "Node",
                Anchor::Module { .. } => "Module",
                Anchor::Crate(_) => "Crate",
                _ => panic!("unexpected anchor {finding:?}"),
            };
            format!(
                "{}:{}\t{anchor}",
                finding.finding.location.path,
                finding.finding.location.line.unwrap_or(0)
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    let mut expected = expected
        .iter()
        .map(|(path, line, anchor)| format!("{path}:{line}\t{anchor}"))
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(expected, actual, "{findings:?}");
}

fn read_fixture_sources(root: &Path, directory: &Path, files: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(directory).expect("fixture source directory") {
        let path = entry.expect("fixture entry").path();
        if path.is_dir() {
            read_fixture_sources(root, &path, files);
        } else {
            files.push((
                path.strip_prefix(root)
                    .expect("relative fixture path")
                    .to_string_lossy()
                    .into_owned(),
                fs::read_to_string(path).expect("fixture source"),
            ));
        }
    }
}

pub(crate) fn assert_placement_fixture(rule: &dyn Rule, name: &str, half: &str) {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/module")
        .join(name)
        .join(half);
    let mut files = Vec::new();
    read_fixture_sources(&directory, &directory.join("src"), &mut files);
    let files = files
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    let context = read_placement_context(&files);
    let expectations =
        fs::read_to_string(directory.join("expected.tsv")).expect("fixture anchor expectations");
    let expectations = expectations
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let columns = line.split('\t').collect::<Vec<_>>();
            assert_eq!(name, columns[0]);
            let (path, line) = columns[1].rsplit_once(':').expect("file:line anchor");
            (path, line.parse::<u32>().expect("line number"), columns[2])
        })
        .collect::<Vec<_>>();
    assert_placement_findings(rule, name, &context, &expectations);
}
