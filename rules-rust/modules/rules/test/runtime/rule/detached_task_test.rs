use crate::check::api::Rule;
use crate::runtime::rule::DetachedTask;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::{assert_code_findings, read_code_context};

#[test]
fn given_dropped_handle_returned_by_a_workspace_function_when_checked_then_reports_each_discard() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn start_worker() -> tokio::task::JoinHandle<()> { tokio::spawn(async {}) }\nfn start_and_drop() {\n    start_worker();\n    let _ = start_worker();\n}\nasync fn start_and_join() {\n    let worker = start_worker();\n    let _ = worker.await;\n}\nfn start_and_hand_over() -> tokio::task::JoinHandle<()> { start_worker() }",
        &[3, 4],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "struct Pool;\nimpl Pool {\n    fn spawn_drain(&self) -> tokio::task::JoinHandle<()> { tokio::spawn(async {}) }\n    fn run(&self) { self.spawn_drain(); }\n}",
        &[4],
    );
}

#[test]
fn given_forgotten_task_handle_when_checked_then_reports_it_like_a_drop() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn run() {\n    let handle = tokio::spawn(async {});\n    std::mem::forget(handle);\n    std::mem::forget(tokio::spawn(async {}));\n    let marker = String::new();\n    std::mem::forget(marker);\n}\nasync fn keep() {\n    let handle = tokio::spawn(async {});\n    let _ = handle.await;\n}",
        &[2, 4],
    );
}

#[test]
fn given_awaited_handle_in_return_value_when_checked_then_does_not_report_detachment() {
    for returned in ["handle.await", "Ok(handle.await?)"] {
        assert_code_findings(
            &DetachedTask,
            "DetachedTask",
            &format!(
                "async fn run() -> Result<(), tokio::task::JoinError> {{\n let handle = tokio::spawn(async {{}});\n return {returned};\n}}"
            ),
            &[],
        );
    }
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn run() -> tokio::task::JoinHandle<()> { return tokio::spawn(async {}); }",
        &[],
    );
}

#[test]
fn given_shadowed_handle_patterns_when_consumed_then_preserves_the_outer_task_identity() {
    for body in [
        "{ let handle = 1u32; consume(handle); }",
        "for handle in [1u32] { consume(handle); }",
        "if let Some(handle) = Some(1u32) { consume(handle); }",
        "match Some(1u32) { Some(handle) => consume(handle), None => {} }",
        "match Some(1u32) { Some(handle) if handle > 0 => consume(handle), _ => {} }",
        "let consume_value = |handle: u32| consume(handle); consume_value(1);",
    ] {
        assert_code_findings(
            &DetachedTask,
            "DetachedTask",
            &format!(
                "fn consume(_: u32) {{}} fn run() {{\n let handle = tokio::spawn(async {{}}); {body}\n}}"
            ),
            &[2],
        );
    }
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn consume(_: u32) {} async fn run() {\n let handle = tokio::spawn(async {}); { let handle = 1u32; consume(handle); } let result = handle.await;\n}",
        &[],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run() { let handle = tokio::spawn(async {}); let handle = handle; let result = handle.await; }",
        &[],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run() { let handle = tokio::spawn(async {}); let compute = |handle: u32| { return handle; }; let value = compute(1); let result = handle.await; }",
        &[],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn run() { let handle = tokio::spawn(async {}); let future = async { return; }; handle.abort(); }",
        &[],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn run() { let future = async {\n let handle = tokio::spawn(async {}); return; }; }",
        &[2],
    );
}

#[test]
fn given_named_handles_when_checked_then_distinguishes_abandonment_from_ownership() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run(ready: bool) {\n let handle = tokio::spawn(async {}); if ready { return; } let result = handle.await;\n}",
        &[2],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn run(ready: bool) {\n let handle = tokio::spawn(async {}); if ready { handle.abort(); }\n}",
        &[2],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn run() {\n let handle = tokio::spawn(async {}); if false { handle.abort(); }\n}",
        &[2],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn run() {\n let handle = tokio::spawn(async {});\n drop(handle);\n let _handle = tokio::spawn(async {});\n}",
        &[2, 4],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run() { let handle = tokio::spawn(async {}); let result = handle.await; }\nfn stop() { let handle = tokio::spawn(async {}); handle.abort(); }\nfn transfer() -> tokio::task::JoinHandle<()> { let handle = tokio::spawn(async {}); handle }",
        &[],
    );
}

#[test]
fn given_async_ownership_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "use tokio::spawn as start;\nasync fn work() {}\nfn run() {\n    start(work());\n    let _ = start(work());\n    drop(start(work()));\n    let owned = start(work());\n}\n",
        &[4, 5, 6, 7],
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
    assert_eq!(4, DetachedTask.check(&context).len());
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

#[test]
fn given_tdd_detached_task_dropped_containers_when_dropped_then_reports_each_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn option_container() {\n    let handle = tokio::spawn(async {});\n    let container = Some(handle);\n    std::mem::drop(container);\n}\nfn boxed_container() {\n    let handle = tokio::spawn(async {});\n    let container = Box::new(handle);\n    std::mem::drop(container);\n}\nfn vector_container() {\n    let handle = tokio::spawn(async {});\n    let container = vec![handle];\n    std::mem::drop(container);\n}\nfn array_container() {\n    let handle = tokio::spawn(async {});\n    let container = [handle];\n    std::mem::drop(container);\n}",
        &[2, 7, 12, 17],
    );
}

#[test]
fn given_tdd_detached_task_array_loop_with_early_break_when_awaited_then_reports_unjoined_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run() {\n    let first = tokio::spawn(async {});\n    let second = tokio::spawn(async {});\n    let handles = [first, second];\n    for handle in handles {\n        let _ = handle.await;\n        break;\n    }\n}",
        &[3],
    );
}

#[test]
fn given_tdd_detached_task_inline_spawn_in_option_when_dropped_then_reports_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run() {\n    let task = Some(tokio::spawn(async {}));\n    drop(task);\n}",
        &[2],
    );
}

#[test]
fn given_inline_spawn_array_handles_are_awaited_when_checked_then_reports_only_orphaned_task() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run() {\n    let handles = [\n        tokio::spawn(async {}),\n        tokio::spawn(async {}),\n    ];\n    for handle in handles {\n        let _ = handle.await;\n    }\n    tokio::spawn(async {});\n}",
        &[9],
    );
}

#[test]
fn given_parenthesized_inline_spawn_array_handles_are_awaited_when_checked_then_reports_only_orphaned_task()
 {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn run() {\n    let handles = [\n        (tokio::spawn(async {})),\n        ((tokio::spawn(async {}))),\n        tokio::spawn(async {}),\n    ];\n    for handle in handles {\n        let _ = handle.await;\n    }\n    tokio::spawn(async {});\n}",
        &[10],
    );
}

#[test]
fn given_task_module_spawn_when_handle_is_dropped_then_reports_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn work() {}\nfn start() {\n    tokio::task::spawn(work());\n}\nasync fn joined() {\n    let handle = tokio::task::spawn(work());\n    let _ = handle.await;\n}",
        &[3],
    );
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "use tokio::task;\nasync fn work() {}\nfn start() {\n    task::spawn(work());\n}",
        &[4],
    );
}

#[test]
fn given_spawn_blocking_when_handle_is_dropped_then_reports_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn start() {\n    tokio::task::spawn_blocking(|| {});\n}\nasync fn joined() {\n    let handle = tokio::task::spawn_blocking(|| {});\n    let _ = handle.await;\n}",
        &[2],
    );
}

#[test]
fn given_runtime_handle_spawn_when_handle_is_dropped_then_reports_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "struct JobRunner {\n    handle: tokio::runtime::Handle,\n}\nimpl JobRunner {\n    fn run(&self) {\n        self.handle.spawn(async {});\n    }\n    fn start(&self) -> tokio::task::JoinHandle<()> {\n        self.handle.spawn(async {})\n    }\n}",
        &[6],
    );
}

#[test]
fn given_aborted_only_handle_in_test_function_when_checked_then_reports_the_spawn() {
    for statement in ["handle.abort();", "let _finished = handle.is_finished();"] {
        assert_code_findings(
            &DetachedTask,
            "DetachedTask",
            &format!(
                "#[tokio::test]\nasync fn given_failing_job_when_started_then_failure_is_visible() {{\n    let handle = tokio::spawn(async {{ panic!(\"job failed\") }});\n    {statement}\n}}\n#[tokio::test]\nasync fn given_aborted_job_when_awaited_then_joins_it() {{\n    let handle = tokio::spawn(async {{}});\n    handle.abort();\n    let joined = handle.await;\n}}\nfn stop() {{\n    let handle = tokio::spawn(async {{}});\n    handle.abort();\n}}"
            ),
            &[3],
        );
    }
}

#[test]
fn given_spawn_local_when_handle_is_dropped_then_reports_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "fn start() {\n    tokio::task::spawn_local(async {});\n}\nfn start_in_set(set: tokio::task::LocalSet) {\n    set.spawn_local(async {});\n}\nasync fn joined() {\n    let handle = tokio::task::spawn_local(async {});\n    let _ = handle.await;\n}",
        &[2, 5],
    );
}

#[test]
fn given_spawn_through_runtime_handle_call_when_handle_is_dropped_then_reports_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "struct Runner;\nimpl Runner { fn spawn<F>(&self, _: F) {} }\nfn start() {\n    tokio::runtime::Handle::current().spawn(async {});\n    tokio::runtime::Handle::current().spawn_blocking(|| {});\n}\nfn start_owned(runtime: tokio::runtime::Runtime) {\n    runtime.handle().spawn(async {});\n}\nfn keep() -> tokio::task::JoinHandle<()> {\n    tokio::runtime::Handle::current().spawn(async {})\n}\nfn unknown(runner: Runner) {\n    runner.spawn(async {});\n}",
        &[4, 5, 8],
    );
}

#[test]
fn given_task_builder_spawn_when_unwrapped_handle_is_dropped_then_reports_each_discard() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn work() {}\nfn start() -> std::io::Result<()> {\n    tokio::task::Builder::new().name(\"job\").spawn(work())?;\n    let _ = tokio::task::Builder::new().spawn(work())?;\n    _ = tokio::task::Builder::new().spawn(work())?;\n    tokio::task::Builder::new().spawn_blocking(|| {}).unwrap();\n    tokio::task::Builder::new().spawn_local(work()).expect(\"spawned\");\n    Ok(())\n}\nasync fn joined() -> std::io::Result<()> {\n    let handle = tokio::task::Builder::new().spawn(work())?;\n    let _ = handle.await;\n    Ok(())\n}\nfn hand_over() -> std::io::Result<tokio::task::JoinHandle<()>> {\n    tokio::task::Builder::new().spawn(work())\n}",
        &[3, 4, 5, 6, 7],
    );
}

#[test]
fn given_handles_assigned_to_places_when_checked_then_reports_only_the_bare_statement() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "struct W {\n    h: Option<tokio::task::JoinHandle<()>>,\n    g: tokio::task::JoinHandle<()>,\n}\nimpl W {\n    fn start(&self) -> tokio::task::JoinHandle<()> { tokio::spawn(async {}) }\n    fn keep_started(&mut self) {\n        self.h = Some(self.start());\n    }\n    fn keep_spawned(&mut self) {\n        let h;\n        h = tokio::spawn(async {});\n        self.h = Some(h);\n    }\n    fn replace(&mut self) {\n        self.g = self.start();\n    }\n    fn detach(&self) {\n        self.start();\n    }\n}",
        &[19],
    );
}

#[test]
fn given_task_builder_spawn_handed_to_drop_when_checked_then_reports_the_spawn() {
    assert_code_findings(
        &DetachedTask,
        "DetachedTask",
        "async fn work() {}\nfn start() -> std::io::Result<()> {\n    drop(tokio::task::Builder::new().spawn(work())?);\n    Ok(())\n}\nasync fn joined() -> std::io::Result<()> {\n    let handle = tokio::task::Builder::new().spawn(work())?;\n    let _ = handle.await;\n    Ok(())\n}",
        &[3],
    );
}
