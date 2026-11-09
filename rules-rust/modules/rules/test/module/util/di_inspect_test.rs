use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::test::declaration::util::read_code_context;

pub(crate) fn read_di_context(source: &str) -> crate::check::model::RuleContext {
    let mut context = read_code_context(source);
    let mut dependencies = context.indexes[0].dependencies.clone();
    dependencies.extend(
        ["thiserror", "tracing", "tracing_subscriber", "lazy_static"]
            .into_iter()
            .map(str::to_owned),
    );
    context.indexes =
        vec![crate::source::syn::CrateIndexBuilder::new(&context.crates[0], dependencies).build()];
    context
}

pub(crate) fn assert_di_lines(rule: &dyn Rule, source: &str, expected: &[u32]) {
    let context = read_di_context(source);
    assert!(
        context
            .indexes
            .iter()
            .all(|index| index.unanalysable.is_empty()),
        "{:?}",
        context.indexes[0].unanalysable
    );
    let findings = rule.check(&context);
    let actual: Vec<_> = findings
        .iter()
        .map(|finding| {
            let Anchor::Node { file, span } = &finding.anchor else {
                panic!("node anchor expected")
            };
            assert_eq!(file, "src/lib.rs");
            assert_eq!(finding.finding.location.line, Some(span.start.line));
            span.start.line
        })
        .collect();
    assert_eq!(expected, actual, "{findings:?}");
}

pub(crate) fn assert_di_fixture(rule: &dyn Rule, name: &str, half: &str) {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code")
        .join(name);
    let source = std::fs::read_to_string(directory.join(format!("phase4-{half}.rs")))
        .expect("persistent source");
    let expected = std::fs::read_to_string(directory.join(format!("phase4-{half}.tsv")))
        .expect("exact expected findings");
    let mut context = read_code_context(&source);
    context.indexes[0].dependencies.push("thiserror".to_owned());
    if name == "MainShape" {
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    }
    assert!(
        context
            .indexes
            .iter()
            .all(|index| index.unanalysable.is_empty())
    );
    let mut actual: Vec<_> = rule
        .check(&context)
        .iter()
        .map(|finding| {
            let Anchor::Node { file, span } = &finding.anchor else {
                panic!("Node expected")
            };
            assert_eq!(&finding.finding.location.path, file);
            assert_eq!(finding.finding.location.line, Some(span.start.line));
            format!(
                "{}\t{}:{}\tNode\t{}",
                finding.finding.rule_id.0, file, span.start.line, finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    let mut expected: Vec<_> = expected
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    expected.sort();
    assert_eq!(expected, actual, "{name}/{half}");
}
