use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::RuleContext;
use crate::config::model::{CrateConfig, Role};
use crate::config::util::parse_project_config;
use crate::source::api::SourceReader;
use crate::source::syn::CrateIndexBuilder;
use crate::source::syn::SynSourceReader;
use crate::test::declaration::util::read_code_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::model::{Dependency, DependencyKind};
use crate::workspace::util::parse_toml_file;
use crate::workspace::util::resolve_cargo_path;

pub(crate) fn assert_import_pair(rule: &dyn Rule, id: &str, good_role: Role) {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code")
        .join(id);
    for (half, role) in [("bad", Role::Context), ("good", good_role)] {
        let source = std::fs::read_to_string(directory.join(format!("phase3-{half}.rs")))
            .expect("persistent source");
        let context = read_import_context(&source, role, &["host"], &["sqlx"], "app");
        let mut actual = rule
            .check(&context)
            .iter()
            .map(|finding| {
                let anchor = match finding.anchor {
                    Anchor::Node { .. } => "Node",
                    Anchor::Module { .. } => "Module",
                    _ => panic!("source anchor required"),
                };
                format!(
                    "{}\t{}:{}\t{}\t{anchor}",
                    finding.finding.rule_id.0,
                    finding.finding.location.path,
                    finding.finding.location.line.expect("line"),
                    finding.finding.subject
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        let expected = std::fs::read_to_string(directory.join(format!("phase3-{half}.tsv")))
            .expect("persistent findings");
        assert_eq!(actual.join("\n"), expected.trim_end(), "{id}/{half}");
    }
}

pub(crate) fn assert_import_workspace(rule: &dyn Rule, expected_name: &str) {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code/PlatformImport/workspace");
    let mut context = read_code_context("");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(&directory)
        .expect("real fixture metadata");
    context.project_config =
        parse_project_config(&context.workspace).expect("real fixture configuration");
    context.crates = SynSourceReader.read_sources(&context.workspace);
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    assert!(
        context
            .indexes
            .iter()
            .all(|index| index.unanalysable.is_empty())
    );
    let mut actual = rule
        .check(&context)
        .iter()
        .map(|finding| {
            assert!(matches!(finding.anchor, Anchor::Node { .. }));
            format!(
                "{}\t{}:{}\t{}\tNode",
                finding.finding.rule_id.0,
                finding.finding.location.path,
                finding.finding.location.line.expect("line"),
                finding.finding.subject
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    let expected =
        std::fs::read_to_string(directory.join(expected_name)).expect("persistent exact findings");
    assert_eq!(actual.join("\n"), expected.trim_end());
}

pub(crate) fn read_import_context(
    source: &str,
    role: Role,
    platforms: &[&str],
    technologies: &[&str],
    product: &str,
) -> RuleContext {
    let mut context = read_code_context(source);
    let words = |words: &[&str]| {
        words
            .iter()
            .map(|word| format!("\"{word}\""))
            .collect::<Vec<_>>()
            .join(",")
    };
    context.workspace.root_manifest = parse_toml_file("Cargo.toml", &format!("[workspace.metadata.klibs-rules]\nproduct-kind=\"{product}\"\nplatform-words=[{}]\ntechnology-words=[{}]\n", words(platforms), words(technologies))).expect("fixture metadata");
    context.workspace.members[0].dependencies = [
        "host-api",
        "sqlx",
        "chrono",
        "time",
        "jiff",
        "tokio",
        "serde",
        "framework",
    ]
    .into_iter()
    .map(|name| Dependency {
        name: name.to_owned(),
        code_name: name.replace('-', "_"),
        path: None,
        kind: DependencyKind::Normal,
        features: Vec::new(),
        is_target_specific: false,
        line: Some(1),
    })
    .collect();
    context.project_config =
        parse_project_config(&context.workspace).expect("valid fixture configuration");
    context.project_config.crates[0] = CrateConfig {
        role: Some(role),
        ..context.project_config.crates[0].clone()
    };
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    assert!(
        context
            .indexes
            .iter()
            .all(|index| index.unanalysable.is_empty()),
        "fixture resolves: {:?}",
        context.indexes[0].unanalysable
    );
    context
}
