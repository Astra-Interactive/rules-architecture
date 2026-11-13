use crate::runtime::rule::LockChoice;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_guard_whose_crossing_await_sits_in_a_macro_argument_when_checked_then_only_the_other_guard_reports()
 {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Source;\nimpl Source { async fn fetch(&self) -> u32 { 1 } }\nasync fn run(gate: tokio::sync::Mutex<Vec<u32>>, source: Source) {\n    let guard = gate.lock().await;\n    tracing::info!(\"{}\", source.fetch().await);\n    let count = guard.len();\n}\nasync fn run_sync(gate: tokio::sync::Mutex<Vec<u32>>) {\n    let guard = gate.lock().await;\n    tracing::info!(\"{}\", guard.len());\n}",
        &[9],
    );
}

#[test]
fn given_temporary_tokio_guard_in_a_let_initializer_or_block_tail_when_checked_then_each_is_reported()
 {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Cache { entries: tokio::sync::Mutex<std::collections::HashMap<u32, String>> }\nasync fn load() -> String { String::new() }\nimpl Cache {\n    async fn count(&self) -> usize {\n        let count = self.entries.lock().await.len();\n        count\n    }\n    async fn get(&self, id: u32) -> Option<String> {\n        self.entries.lock().await.get(&id).cloned()\n    }\n    async fn get_loaded(&self, id: u32) -> Option<String> {\n        let loaded = self.entries.lock().await.insert(id, load().await);\n        loaded\n    }\n}",
        &[5, 9],
    );
}

#[test]
fn given_dropped_guard_or_unreachable_await_when_checked_then_requires_actual_guard_crossing() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "async fn run(gate: tokio::sync::Mutex<Vec<u32>>) {\n let guard = gate.lock().await; drop(guard); tokio::time::sleep(std::time::Duration::ZERO).await;\n}",
        &[2],
    );
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "async fn run(gate: tokio::sync::Mutex<Vec<u32>>) {\n let guard = gate.lock().await; if false { tokio::time::sleep(std::time::Duration::ZERO).await; }\n}",
        &[2],
    );
}

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Shared { flag: tokio::sync::Mutex<bool> }\n",
        &[1],
    );
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Mutex<T>(T);\nstruct Shared { flag: Mutex<bool> }\n",
        &[],
    );
}

#[test]
fn given_tokio_guard_scopes_when_checked_then_requires_an_await_crossing_the_guard() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "async fn run(gate: tokio::sync::Mutex<Vec<u32>>) {\n    let guard = gate.lock().await;\n    let count = guard.len();\n}\n",
        &[2],
    );
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "async fn run(gate: tokio::sync::Mutex<Vec<u32>>) {\n    let guard = gate.lock().await;\n    tokio::time::sleep(std::time::Duration::ZERO).await;\n    let count = guard.len();\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&LockChoice, "LockChoice", "bad");
    assert_code_fixture(&LockChoice, "LockChoice", "good");
}

#[test]
fn given_lock_choice_typed_guard_binding_when_linted_then_reports_the_local() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "async fn run(gate: tokio::sync::Mutex<Vec<u32>>) {\n    let guard: tokio::sync::MutexGuard<'_, Vec<u32>> = gate.lock().await;\n    let count = guard.len();\n}\nasync fn run_crossing(gate: tokio::sync::Mutex<Vec<u32>>) {\n    let guard: tokio::sync::MutexGuard<'_, Vec<u32>> = gate.lock().await;\n    tokio::time::sleep(std::time::Duration::ZERO).await;\n    let count = guard.len();\n}",
        &[2],
    );
}

#[test]
fn given_lock_choice_tokio_rwlock_guards_without_crossing_await_when_linted_then_reports_both() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Cache { entries: tokio::sync::RwLock<std::collections::HashMap<u32, String>> }\nimpl Cache {\n    async fn put(&self, id: u32, value: String) {\n        let mut entries = self.entries.write().await;\n        entries.insert(id, value);\n    }\n    async fn get(&self, id: u32) -> Option<String> {\n        let entries = self.entries.read().await;\n        entries.get(&id).cloned()\n    }\n}",
        &[4, 8],
    );
}

#[test]
fn given_lock_choice_temporary_tokio_guard_when_linted_then_reports_the_statement() {
    assert_code_findings(
        &LockChoice,
        "LockChoice",
        "struct Cache { cache: tokio::sync::Mutex<std::collections::HashMap<u32, String>> }\nasync fn load() -> String { String::new() }\nimpl Cache {\n    async fn put(&self, id: u32, value: String) {\n        self.cache.lock().await.insert(id, value);\n    }\n    async fn put_loaded(&self, id: u32) {\n        self.cache.lock().await.insert(id, load().await);\n    }\n}",
        &[5],
    );
}
