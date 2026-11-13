use crate::check::api::Rule;
use crate::runtime::rule::CallbackBridgeShape;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};
use crate::workspace::model::{Dependency, DependencyKind};

#[test]
fn given_registration_guard_dropped_by_a_statement_when_checked_then_reports_each_discard() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Guard; impl Drop for Guard { fn drop(&mut self) {} } impl Guard { fn register() -> Self { Self } fn id(&self) -> u32 { 1 } }\nfn run() { Guard::register(); }\nfn assign() { _ = Guard::register(); }\nfn keep() { let registration = Guard::register(); drop(registration); }\nfn nested() { Guard::register().id(); }",
        &[2, 3],
    );
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "struct Guard; impl Drop for Guard { fn drop(&mut self) {} } impl Guard { fn register() -> u32 { 1 } }\nfn run() { Guard::register(); }",
        &[],
    );
}

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
fn given_boxed_closure_passed_to_external_method_when_spawning_then_reports_spawn() {
    assert_code_findings(
        &CallbackBridgeShape,
        "CallbackBridgeShape",
        "fn run(player: &vendor::Player) {\n    player.add_state_listener(Box::new(move |state: u8| {\n        tokio::spawn(async move { let _ = state; });\n    }));\n    player.map_state(|state: u8| {\n        tokio::spawn(async move { let _ = state; });\n    });\n}",
        &[3],
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

fn assert_tdd_callback_bridge_external_library_finding(source: &str, expected_line: u32) {
    let directory = tempfile::tempdir().expect("temporary callback dependency");
    std::fs::create_dir_all(directory.path().join("src")).expect("dependency source directory");
    std::fs::write(
        directory.path().join("Cargo.toml"),
        "[package]\nname = \"callback-library\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("dependency manifest");
    std::fs::write(
        directory.path().join("src/lib.rs"),
        "pub struct Library { handler: Option<Box<dyn Fn(u8) + Send + Sync>> } impl Library { pub fn register(&mut self, handler: impl Fn(u8) + Send + Sync + 'static) { self.handler = Some(Box::new(handler)); } }",
    )
    .expect("dependency source");

    let mut context = read_code_context(source);
    let owner_name = context.crates[0].crate_name.clone();
    let owner = context
        .workspace
        .members
        .iter_mut()
        .find(|member| member.name == owner_name)
        .expect("source crate workspace member");
    owner
        .dependencies
        .retain(|dependency| dependency.code_name != "vendor");
    owner.dependencies.push(Dependency {
        name: "callback-library".to_owned(),
        code_name: "vendor".to_owned(),
        path: Some(directory.path().to_string_lossy().into_owned()),
        kind: DependencyKind::Normal,
        features: Vec::new(),
        is_target_specific: false,
        line: None,
    });
    context.indexes[0]
        .dependency_packages
        .insert("vendor".to_owned(), "callback_library".to_owned());
    context.names = std::sync::OnceLock::new();

    let findings = CallbackBridgeShape.check(&context);
    assert_eq!(
        1,
        findings.len(),
        "source: {source}\nfindings: {findings:?}"
    );
    let finding = &findings[0];
    assert_eq!("CallbackBridgeShape", finding.finding.rule_id.0);
    assert_eq!(Some(expected_line), finding.finding.location.line);
    assert!(matches!(
        &finding.anchor,
        crate::check::model::Anchor::Node { .. }
    ));
}

#[test]
fn given_tdd_callback_bridge_external_library_blocking_send_when_registered_then_reports_call() {
    assert_tdd_callback_bridge_external_library_finding(
        "use vendor::Library;\nfn run(library: &mut Library, sender: tokio::sync::mpsc::Sender<u8>) {\n    library.register(move |event| { let _ = sender.blocking_send(event); });\n}",
        3,
    );
}

#[test]
fn given_tdd_callback_bridge_external_library_per_event_spawn_when_registered_then_reports_call() {
    assert_tdd_callback_bridge_external_library_finding(
        "use vendor::Library;\nfn run(library: &mut Library) {\n    library.register(move |event| {\n        let task = tokio::spawn(async move { let _ = event; });\n        task.abort();\n    });\n}",
        4,
    );
}

#[test]
fn given_task_started_per_event_through_a_handle_or_the_blocking_pool_when_registered_then_reports_call()
 {
    for spawn in [
        "handle.spawn(async move { let _ = event; })",
        "handle.spawn_blocking(move || { let _ = event; })",
        "tokio::task::spawn_blocking(move || { let _ = event; })",
        "tokio::task::spawn_local(async move { let _ = event; })",
    ] {
        assert_tdd_callback_bridge_external_library_finding(
            &format!(
                "use vendor::Library;\nfn run(library: &mut Library, handle: tokio::runtime::Handle) {{\n    library.register(move |event| {{\n        let task = {spawn};\n        drop(task);\n    }});\n}}"
            ),
            4,
        );
    }
}
