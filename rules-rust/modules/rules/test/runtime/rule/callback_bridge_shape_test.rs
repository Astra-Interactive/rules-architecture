use crate::check::api::Rule;
use crate::runtime::rule::CallbackBridgeShape;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};

#[test]
fn given_temporarily_stored_callable_when_consumed_synchronously_then_is_not_a_registration() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Library { callback: Option<Box<dyn Fn()>> } impl Library { fn execute(mut self, callback: impl Fn() + 'static) { self.callback = Some(Box::new(callback)); } }\nfn run(library: Library) { library.execute(|| { let handle = tokio::spawn(async {}); }); }",
        &[],
    );
    for body in [
        "let mut temporary = Holder { callback: None }; temporary.callback = Some(Box::new(callback));",
        "Holder { callback: Some(Box::new(callback)) };",
        "let callback = || {}; return Holder { callback: Some(Box::new(callback)) };",
    ] {
        assert_code_findings(
            &CallbackBridgeShape,
            "CallbackBridgeShape",
            &format!(
                "struct Holder {{ callback: Option<Box<dyn Fn()>> }} fn apply(callback: impl Fn() + 'static) {{ {body} }}\nfn run() {{ apply(|| {{ let handle = tokio::spawn(async {{}}); }}); }}"
            ),
            &[],
        );
    }
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Library { callback: Option<Box<dyn FnOnce()>> } impl Library { fn execute<F: FnOnce() + 'static>(&mut self, callback: F) { self.callback = Some(Box::new(callback)); self.callback.take().unwrap()(); } }\nfn run(library: &mut Library) { library.execute(|| { let handle = tokio::spawn(async {}); }); }",
        &[],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Registration<F> { callback: F } impl<F: FnOnce()> Registration<F> { fn invoke(self) { (self.callback)(); } } fn apply(callback: impl FnOnce()) { Registration { callback }.invoke(); }\nfn run() { apply(|| { let handle = tokio::spawn(async {}); }); }",
        &[],
    );
}

#[test]
fn given_synchronous_callable_contracts_when_checked_then_does_not_treat_them_as_registrations() {
    for body in [
        "std::array::from_fn::<_, 2, _>(|_| tokio::spawn(async {}))",
        "Some(1).map(|_| tokio::spawn(async {}))",
        "[1, 2].into_iter().map(|_| tokio::spawn(async {})).collect::<Vec<_>>()",
        "apply(|| tokio::spawn(async {}))",
    ] {
        assert_code_findings(
            &CallbackBridgeShape,
            "CallbackBridgeShape",
            &format!(
                "fn apply<F: FnOnce() -> T, T>(callback: F) -> T {{ callback() }}\nfn run() -> impl Sized {{ {body} }}"
            ),
            &[],
        );
    }
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "fn unknown(callback: impl Fn()) {}\nfn run() { unknown(|| { tokio::spawn(async {}); }); }",
        &[],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Value { ready: bool } fn apply(callback: impl FnOnce() -> bool) -> bool { callback() } impl Value { fn update(&mut self, callback: impl FnOnce() -> bool) { self.ready = apply(callback); } }\nfn run(value: &mut Value) { value.update(|| { let owned = tokio::spawn(async {}); true }); }",
        &[],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Library { callback: Option<Box<dyn Fn()>> } impl Library { #[cfg(any())] fn register(&mut self, callback: impl Fn() + 'static) { self.callback = Some(Box::new(callback)); } #[cfg(not(any()))] fn register(&mut self, callback: impl Fn()) { callback(); } }\nfn run(library: &mut Library) { library.register(|| { tokio::spawn(async {}); }); }",
        &[],
    );
}

#[test]
fn given_stored_library_callback_when_sending_then_rejects_blocking_send_only_in_that_callback() {
    let registration = "struct Library { callback: Option<Box<dyn Fn()>> } impl Library { fn register(&mut self, callback: impl Fn() + 'static) { self.callback = Some(Box::new(callback)); } }";
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        &format!(
            "{registration}\nfn run(library: &mut Library) {{\n let (sender, receiver) = tokio::sync::mpsc::channel::<u32>(1);\n let callback = move || {{ sender.blocking_send(1); }};\n let sender = 1;\n library.register(callback);\n}}"
        ),
        &[4],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        &format!(
            "{registration}\nfn run(library: &mut Library, sender: tokio::sync::mpsc::Sender<u32>) {{\n library.register(move || {{ sender.blocking_send(1); }});\n}}"
        ),
        &[3],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        &format!(
            "{registration}\nfn run(library: &mut Library, sender: tokio::sync::mpsc::Sender<u32>) {{\n sender.blocking_send(1);\n library.register(move || {{ sender.try_send(1); }});\n}}"
        ),
        &[],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        &format!(
            "{registration}\nstruct Other; impl Other {{ fn blocking_send(&self, _: u32) {{}} }}\nfn run(library: &mut Library, sender: Other) {{\n library.register(move || {{ sender.blocking_send(1); }});\n}}"
        ),
        &[],
    );
}

#[test]
fn given_shadowed_callback_capture_when_registered_then_uses_declaration_environment() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Registration<F> { callback: F } fn register<F: Fn()>(callback: F) -> Registration<F> { Registration { callback } } fn run() { let callback = || {}; let callback = || { register(callback); }; register(callback); }",
        &[],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Registration<F> { callback: F } fn register<F: Fn()>(callback: F) -> Registration<F> { Registration { callback } } fn run() {\n let callback = || { tokio::spawn(async {}); };\n let callback = || { register(callback); }; register(callback);\n}",
        &[2],
    );
}

#[test]
fn given_guard_factory_when_discarded_then_uses_its_owned_return_contract() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Guard; impl Drop for Guard { fn drop(&mut self) {} } impl Guard { fn register() -> Self { Self } }\nfn run() { let _ = Guard::register(); }",
        &[2],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Guard; impl Drop for Guard { fn drop(&mut self) {} } impl Guard { fn register() -> u32 { 1 } }\nfn run() { let _ = Guard::register(); }",
        &[],
    );
}

#[test]
fn given_bound_callback_when_registered_then_checks_its_task_creation() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Registration<F> { callback: F } fn register<F: Fn()>(callback: F) -> Registration<F> { Registration { callback } } fn run() {\n let callback = || { tokio::spawn(async {}); };\n register(callback);\n}",
        &[2],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Registration<F> { callback: F } fn register<F: Fn()>(callback: F) -> Registration<F> { Registration { callback } } fn run() {\n let callback = || { let value = 1; };\n register(callback);\n}",
        &[],
    );
}

#[test]
fn given_async_ownership_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Registration<F> { callback: F } fn consume<F: Fn()>(callback: F) -> Registration<F> { Registration { callback } }\nfn run() { consume(|| { tokio::spawn(async {}); }); }\n",
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
    let source = "struct Registration<F> { callback: F } fn consume<F: Fn()>(callback: F) -> Registration<F> { Registration { callback } }\nfn run() { consume(|| { tokio::spawn(async {}); }); }\n";
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
