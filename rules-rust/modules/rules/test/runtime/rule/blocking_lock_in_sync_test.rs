use crate::runtime::rule::BlockingLockInSync;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_unrelated_generic_container_and_shadowed_names_when_checked_then_keeps_type_identity() {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Container<T>(T);\nimpl<T> Container<T> { fn blocking_lock(&self) {} }\nstruct Owner;\nimpl Owner { fn on_reload(&self, gate: Container<tokio::sync::Mutex<()>>) { gate.blocking_lock(); } }",
        &[],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Local;\nimpl Local { fn blocking_lock(&self) {} }\nstruct Owner;\nimpl Owner { fn on_reload(&self, gate: tokio::sync::Mutex<()>) { let gate = Local; gate.blocking_lock(); } }",
        &[],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "#[cfg(unix)] type Gate = Local;\n#[cfg(not(unix))] type Gate = tokio::sync::Mutex<()>;\nstruct Local;\nimpl Local { fn blocking_lock(&self) {} }\nstruct Owner;\nimpl Owner { #[cfg(unix)] fn on_reload(&self, gate: Gate) { gate.blocking_lock(); } }",
        &[],
    );
}

#[test]
fn given_typed_runtime_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "type Gate = tokio::sync::Mutex<u32>;\nstruct Owner { gate: Gate }\nimpl Owner { fn on_reload(&self) { self.gate.blocking_lock(); } }\n",
        &[3],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Mutex;\nimpl Mutex { fn blocking_lock(&self) {} }\nstruct Owner { gate: Mutex }\nimpl Owner { fn on_reload(&self) { self.gate.blocking_lock(); } }\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&BlockingLockInSync, "BlockingLockInSync", "bad");
    assert_code_fixture(&BlockingLockInSync, "BlockingLockInSync", "good");
}

#[test]
fn given_blocking_calls_in_on_enable_and_rwlock_forms_when_checked_then_reports_each_call() {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Owner { gate: tokio::sync::Mutex<u32>, entries: tokio::sync::RwLock<u32> }\nimpl Owner {\n    fn on_enable(&self) { self.gate.blocking_lock(); }\n}\nimpl Drop for Owner {\n    fn drop(&mut self) { self.entries.blocking_read(); }\n}",
        &[3, 6],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Owner { entries: tokio::sync::RwLock<u32> }\nimpl Owner {\n    fn on_enable(&self) { self.entries.blocking_write(); }\n    fn on_disable(&self) { self.entries.blocking_read(); }\n    fn on_reload(&self) { self.entries.blocking_write(); }\n}",
        &[3, 5],
    );
}

#[test]
fn given_blocking_locks_in_on_reload_on_disable_and_a_free_function_when_checked_then_reports_only_on_reload()
 {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Owner { gate: tokio::sync::Mutex<u32>, entries: tokio::sync::RwLock<u32> }\nimpl Owner {\n    fn on_reload(&self) {\n        self.gate.blocking_lock();\n        self.entries.blocking_read();\n        self.entries.blocking_write();\n    }\n    fn on_disable(&self) {\n        self.gate.blocking_lock();\n        self.entries.blocking_read();\n        self.entries.blocking_write();\n    }\n}\nfn on_reload(gate: tokio::sync::Mutex<u32>) {\n    gate.blocking_lock();\n}",
        &[4, 5, 6],
    );
}

#[test]
fn given_blocking_channel_calls_in_drop_when_checked_then_reports_each_but_not_try_recv_or_join() {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Worker {\n    std_rx: std::sync::mpsc::Receiver<u32>,\n    rx: tokio::sync::mpsc::Receiver<u32>,\n    events: tokio::sync::broadcast::Receiver<u32>,\n    tx: tokio::sync::mpsc::Sender<u32>,\n    handle: Option<std::thread::JoinHandle<()>>,\n}\nimpl Drop for Worker {\n    fn drop(&mut self) {\n        self.std_rx.recv();\n        self.std_rx.recv_timeout(std::time::Duration::from_secs(1));\n        self.rx.blocking_recv();\n        self.events.blocking_recv();\n        self.tx.blocking_send(1);\n        self.std_rx.try_recv();\n        if let Some(handle) = self.handle.take() {\n            let _ = handle.join();\n        }\n    }\n}\nimpl Worker {\n    fn close(&mut self) { self.std_rx.recv(); }\n}",
        &[10, 11, 12, 13, 14],
    );
}

#[test]
fn given_blocking_recv_on_an_unbounded_receiver_in_drop_when_checked_then_reports_the_call() {
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "struct Worker { rx: tokio::sync::mpsc::UnboundedReceiver<u32> }\nimpl Drop for Worker {\n    fn drop(&mut self) { self.rx.blocking_recv(); }\n}",
        &[3],
    );
    assert_code_findings(
        &BlockingLockInSync,
        "BlockingLockInSync",
        "use tokio::sync::mpsc::UnboundedReceiver;\nstruct Worker { rx: UnboundedReceiver<u32> }\nimpl Drop for Worker {\n    fn drop(&mut self) { self.rx.blocking_recv(); }\n}",
        &[4],
    );
}
