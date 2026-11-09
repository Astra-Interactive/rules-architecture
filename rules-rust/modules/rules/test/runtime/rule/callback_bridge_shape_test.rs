use crate::check::api::Rule;
use crate::runtime::rule::CallbackBridgeShape;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};

#[test]
fn given_async_ownership_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "fn consume<F: Fn()>(_: F) {}\nfn run() { consume(|| { tokio::spawn(async {}); }); }\n",
        &[2],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "fn consume<F: Fn()>(_: F) {}\nfn spawn() {}\nfn run() { consume(|| spawn()); let handle = tokio::spawn(async {}); }\n",
        &[],
    );
}

#[test]
fn given_persistent_cases_when_checked_then_rejects_unexpected_unresolved_source() {
    let source =
        "fn consume<F: Fn()>(_: F) {}\nfn run() { consume(|| { tokio::spawn(async {}); }); }\n";
    let context = read_code_context(source);
    for index in &context.indexes {
        assert!(index.unanalysable.is_empty(), "{:?}", index.unanalysable);
    }
    assert_eq!(1, CallbackBridgeShape.check(&context).len());
}

#[test]
fn given_drop_owner_factories_when_checked_then_requires_the_actual_drop_return_type() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Guard;\nimpl Drop for Guard { fn drop(&mut self) {} }\nimpl Guard { fn new() -> Self { Self } }\nfn run() { let _ = Guard::new(); }\n",
        &[4],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Guard;\nimpl Drop for Guard { fn drop(&mut self) {} }\nimpl Guard { fn new() -> u32 { 0 } }\nfn run() { let _ = Guard::new(); }\n",
        &[],
    );
}

#[test]
fn given_synchronous_library_builder_when_checked_then_keeps_callback_intent_in_review() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "fn run(sender: tokio::sync::mpsc::Sender<u32>) {\n    let values = std::array::from_fn::<_, 2, _>(|_| sender.blocking_send(1));\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&CallbackBridgeShape, "CallbackBridgeShape", "bad");
    assert_code_fixture(&CallbackBridgeShape, "CallbackBridgeShape", "good");
}

#[test]
fn given_constructor_returning_a_borrowed_guard_when_checked_then_preserves_the_owned_guard() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Guard;\nstatic GUARD: Guard = Guard;\nimpl Drop for Guard { fn drop(&mut self) {} }\nimpl Guard { fn new() -> &'static Guard { &GUARD } }\nfn run() { let _ = Guard::new(); }",
        &[],
    );
}

#[test]
fn given_actual_good_stream_and_bridge_when_checked_then_keeps_owned_stream_closures() {
    use crate::config::util::parse_project_config;
    use crate::source::api::SourceReader;
    use crate::source::syn::{CrateIndexBuilder, SynSourceReader};
    use crate::workspace::api::WorkspaceReader;
    use crate::workspace::cargo::CargoWorkspaceReader;
    use crate::workspace::util::resolve_cargo_path;
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cases/doc");
    let mut context = read_code_context("");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(&directory)
        .expect("document workspace");
    context.project_config = parse_project_config(&context.workspace).expect("document config");
    context.crates = SynSourceReader.read_sources(&context.workspace);
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    let endings = ["in_memory_timer.rs", "acme_playback_bridge.rs"];
    for ending in endings {
        assert!(
            context
                .crates
                .iter()
                .flat_map(|sources| &sources.files)
                .any(|file| file.path.contains("modules/good/") && file.path.ends_with(ending))
        );
        assert!(!context.indexes.iter().flat_map(|index| &index.unanalysable).any(|source| matches!(&source.site, crate::source::model::UnanalysableSite::Node { file, .. } if file.contains("modules/good/") && file.ends_with(ending))), "unexpected unresolved Good {ending}");
    }
    let findings = CallbackBridgeShape.check(&context);
    assert!(
        !findings.iter().any(
            |finding| finding.finding.location.path.contains("modules/good/")
                && endings
                    .iter()
                    .any(|ending| finding.finding.location.path.ends_with(ending))
        ),
        "{findings:?}"
    );
}
