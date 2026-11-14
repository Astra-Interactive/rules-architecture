use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::source::model::SourceTree;
use crate::source::syn::CrateIndexBuilder;
use crate::source::syn::ModuleTreeWalker;
use crate::test::check::usecase::read_case_context;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FindingMessage {
    pub(crate) line: u32,
    pub(crate) message: String,
}

pub(crate) fn list_finding_messages(findings: &[RuleFinding]) -> Vec<FindingMessage> {
    let mut messages: Vec<FindingMessage> = findings
        .iter()
        .map(|finding| FindingMessage {
            line: finding.finding.location.line.expect("finding line"),
            message: finding.finding.message.clone(),
        })
        .collect();
    messages.sort();
    messages
}

pub(crate) fn read_code_context(source: &str) -> RuleContext {
    let mut context = read_case_context("ImplementationName", "pass-syntax");
    let directory = tempfile::tempdir().expect("temporary source directory");
    fs::create_dir(directory.path().join("src")).expect("source directory");
    fs::write(directory.path().join("src/lib.rs"), source).expect("source file");
    let crate_name = context.crates[0].crate_name.clone();
    let target = context.crates[0].target.clone();
    context.crates = vec![
        ModuleTreeWalker::new(directory.path().to_owned(), crate_name, target)
            .walk_crate("src/lib.rs", SourceTree::Src),
    ];
    let dependencies = [
        "tokio",
        "futures",
        "futures_executor",
        "async_trait",
        "serde",
        "parking_lot",
        "tracing",
        "log",
        "uuid",
        "vendor",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    context.indexes = context
        .crates
        .iter()
        .map(|sources| CrateIndexBuilder::new(sources, dependencies.clone()).build())
        .collect();
    context
}

pub(crate) fn assert_code_findings(rule: &dyn Rule, rule_name: &str, source: &str, lines: &[u32]) {
    let context = read_code_context(source);
    assert_runtime_resolution(&context, rule_name);
    let findings = rule.check(&context);
    let mut actual = findings
        .iter()
        .map(|finding| {
            assert_eq!(rule_name, finding.finding.rule_id.0);
            assert_eq!("src/lib.rs", finding.finding.location.path);
            let Anchor::Node { file, span } = &finding.anchor else {
                panic!("expected exact node anchor: {finding:?}");
            };
            assert_eq!("src/lib.rs", file);
            assert_eq!(Some(span.start.line), finding.finding.location.line);
            span.start.line
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(lines, actual, "source: {source}\nfindings: {findings:?}");
}

fn assert_resolved_fixture(context: &RuleContext, rule_name: &str, half: &str) {
    assert_runtime_resolution(context, rule_name);
    if [
        "ImplementationName",
        "RolelessTypeName",
        "NamespaceType",
        "StructDefaultValue",
        "ContractShape",
        "BlockOnInTest",
        "BlockOnOutsideBoundary",
        "ErrorPlacement",
        "EnumFamilyFile",
        "FakePlacement",
        "InMemoryPlacement",
        "ImplPlacement",
        "UnsafePlacement",
        "ListenerName",
        "EventPlacement",
        "VisibilityTier",
        "AssociatedFactoryFunction",
        "ModelContainsFunctions",
        "ModelShouldBeImmutable",
    ]
    .contains(&rule_name)
    {
        for index in &context.indexes {
            assert!(
                index.unanalysable.is_empty(),
                "unexpected unresolved source in {rule_name}/{half}: {:?}",
                index.unanalysable
            );
        }
    }
}

fn assert_runtime_resolution(context: &RuleContext, rule_name: &str) {
    if ![
        "DetachedTask",
        "WatchTimeout",
        "ImpureUpdateClosure",
        "CallbackBridgeShape",
        "BridgeCleanup",
        "ThreadPrimitive",
        "DurationOverloadCall",
        "ResultConsumption",
        "TimePrimitive",
        "ErrorLogging",
        "CatchUnwind",
        "BlockingCallInAsync",
        "BroadcastLag",
        "SelectCancelSafety",
        "JoinSetBehindLock",
        "BlockingLockInSync",
        "CriticalSectionWrite",
        "LockChoice",
        "TestTimePaused",
        "ModelRuleTest",
    ]
    .contains(&rule_name)
    {
        return;
    }
    for sources in &context.crates {
        assert!(
            sources.unanalysable.is_empty(),
            "{rule_name}: {:?}",
            sources.unanalysable
        );
    }
    for index in &context.indexes {
        assert!(
            index.unanalysable.is_empty(),
            "{rule_name}: {:?}",
            index.unanalysable
        );
    }
}

fn find_fixture_directory(rule_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code")
        .join(rule_name)
}

fn read_fixture_context(rule_name: &str, half: &str) -> RuleContext {
    let directory = find_fixture_directory(rule_name);
    let source = fs::read_to_string(directory.join(format!("{half}.rs")))
        .expect("persistent source fixture");
    let mut context = read_code_context(&source);
    let manifest = directory.join("manifest.toml");
    if manifest.exists() {
        let text = fs::read_to_string(&manifest).expect("persistent manifest fixture");
        context.workspace.members[0].manifest =
            crate::workspace::util::parse_toml_file(&manifest.to_string_lossy(), &text)
                .expect("valid manifest fixture");
    }
    context
}

pub(crate) fn read_fixture_messages(rule: &dyn Rule, rule_name: &str) -> Vec<FindingMessage> {
    let context = read_fixture_context(rule_name, "bad");
    list_finding_messages(&rule.check(&context))
}

pub(crate) fn assert_code_fixture(rule: &dyn Rule, rule_name: &str, half: &str) {
    let expected =
        fs::read_to_string(find_fixture_directory(rule_name).join(format!("{half}.tsv")))
            .expect("persistent anchor fixture");
    let context = read_fixture_context(rule_name, half);
    assert_resolved_fixture(&context, rule_name, half);
    let findings = rule.check(&context);
    let mut anchors = findings
        .iter()
        .map(|finding| {
            let Anchor::Node { file, span } = &finding.anchor else {
                panic!("expected node anchor: {finding:?}");
            };
            assert_eq!(file, &finding.finding.location.path);
            assert_eq!(Some(span.start.line), finding.finding.location.line);
            format!(
                "{}\t{}:{}\tNode",
                finding.finding.rule_id.0, file, span.start.line
            )
        })
        .collect::<Vec<_>>();
    anchors.sort();
    let mut expected = expected
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(expected, anchors, "persistent fixture {rule_name}/{half}");
}
