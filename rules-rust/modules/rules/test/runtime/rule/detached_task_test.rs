use crate::check::api::Rule;
use crate::runtime::rule::DetachedTask;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};

#[test]
fn given_async_ownership_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "use tokio::spawn as start;\nasync fn work() {}\nfn run() {\n    start(work());\n    let _ = start(work());\n    drop(start(work()));\n    let owned = start(work());\n}\n",
        &[4, 5, 6],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn spawn<T>(_: T) {}\nfn run() { spawn(1); let _ = spawn(2); }\n",
        &[],
    );
}

#[test]
fn given_persistent_cases_when_checked_then_rejects_unexpected_unresolved_source() {
    let source = "use tokio::spawn as start;\nasync fn work() {}\nfn run() {\n    start(work());\n    let _ = start(work());\n    drop(start(work()));\n    let owned = start(work());\n}\n";
    let context = read_code_context(source);
    for index in &context.indexes {
        assert!(index.unanalysable.is_empty(), "{:?}", index.unanalysable);
    }
    assert_eq!(3, DetachedTask.check(&context).len());
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&DetachedTask, "DetachedTask", "bad");
    assert_code_fixture(&DetachedTask, "DetachedTask", "good");
}

#[test]
fn given_real_cargo_rename_and_workspace_reexports_when_checked_then_preserves_external_identity() {
    use crate::config::util::parse_project_config;
    use crate::runtime::rule::BlockingLockInSync;
    use crate::source::api::SourceReader;
    use crate::source::syn::{CrateIndexBuilder, SynSourceReader};
    use crate::workspace::api::WorkspaceReader;
    use crate::workspace::cargo::CargoWorkspaceReader;
    use crate::workspace::util::resolve_cargo_path;
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code/DetachedTask/workspace");
    let mut context = read_code_context("");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(&directory)
        .expect("actual workspace");
    context.project_config = parse_project_config(&context.workspace).expect("configuration");
    context.crates = SynSourceReader.read_sources(&context.workspace);
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    for index in &context.indexes {
        assert!(index.unanalysable.is_empty(), "{:?}", index.unanalysable);
    }
    for (rule, line) in [
        (&DetachedTask as &dyn Rule, 3),
        (&BlockingLockInSync as &dyn Rule, 6),
    ] {
        let findings = rule.check(&context);
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(
            "modules/consumer/src/lib.rs",
            findings[0].finding.location.path
        );
        assert_eq!(Some(line), findings[0].finding.location.line);
        assert!(matches!(
            findings[0].anchor,
            crate::check::model::Anchor::Node { .. }
        ));
    }
}
