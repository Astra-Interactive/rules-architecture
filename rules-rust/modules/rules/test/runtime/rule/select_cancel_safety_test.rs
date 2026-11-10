use crate::runtime::rule::SelectCancelSafety;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "async fn run(gate: tokio::sync::Mutex<()>) {\n    loop { tokio::select! { _guard = gate.lock() => {}, else => {} } }\n}\n",
        &[2],
    );
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "async fn run(gate: tokio::sync::Mutex<()>) {\n    let future = gate.lock();\n    tokio::pin!(future);\n    loop { tokio::select! { _guard = &mut future => {}, else => {} } }\n}\n",
        &[],
    );
}

#[test]
fn given_read_exact_on_connected_local_in_looping_select_when_checked_then_reports_the_branch() {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "use tokio::io::AsyncReadExt;\nuse tokio::net::TcpStream;\nasync fn pump(address: &str, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    let Ok(mut stream) = TcpStream::connect(address).await else {\n        return;\n    };\n    let mut buffer = [0u8; 4];\n    loop {\n        tokio::select! { read = stream.read_exact(&mut buffer) => { let _ = read; }, _ = shutdown.changed() => return, }\n    }\n}\nasync fn typed(stream: &mut TcpStream, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    let mut buffer = [0u8; 4];\n    loop {\n        tokio::select! { read = stream.read_exact(&mut buffer) => { let _ = read; }, _ = shutdown.changed() => return, }\n    }\n}",
        &[9, 15],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&SelectCancelSafety, "SelectCancelSafety", "bad");
    assert_code_fixture(&SelectCancelSafety, "SelectCancelSafety", "good");
}

#[test]
fn given_queued_or_lossy_futures_in_looping_select_when_checked_then_reports_each_branch() {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "use tokio::io::AsyncReadExt;\nstruct Owner { state: tokio::sync::RwLock<u32>, permits: tokio::sync::Semaphore, signal: tokio::sync::Notify }\nimpl Owner {\n    async fn write(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n        loop { tokio::select! { guard = self.state.write() => { drop(guard); }, _ = shutdown.changed() => break, } }\n    }\n    async fn read(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n        loop { tokio::select! { guard = self.state.read() => { drop(guard); }, _ = shutdown.changed() => break, } }\n    }\n    async fn acquire(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n        loop { tokio::select! { permit = self.permits.acquire() => { drop(permit); }, _ = shutdown.changed() => break, } }\n    }\n    async fn wait(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n        loop { tokio::select! { _ = self.signal.notified() => {}, _ = shutdown.changed() => break, } }\n    }\n}\nasync fn text(stream: &mut tokio::net::TcpStream, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    let mut text = String::new();\n    loop { tokio::select! { read = stream.read_to_string(&mut text) => { drop(read); }, _ = shutdown.changed() => return, } }\n}",
        &[5, 8, 11, 14, 19],
    );
}

#[test]
fn given_awaited_connection_bound_through_try_or_unwrap_when_looping_select_then_reports_branch() {
    for connect in [
        "TcpStream::connect(address).await?",
        "TcpStream::connect(address).await.unwrap()",
        "tokio::fs::File::open(address).await?",
    ] {
        assert_code_findings(
            &SelectCancelSafety,
            "SelectCancelSafety",
            &format!(
                "use tokio::io::AsyncReadExt;\nuse tokio::net::TcpStream;\nasync fn pump(address: &str, mut shutdown: tokio::sync::watch::Receiver<bool>) -> std::io::Result<()> {{\n    let mut stream = {connect};\n    let mut buffer = [0u8; 4];\n    loop {{\n        tokio::select! {{ read = stream.read_exact(&mut buffer) => {{ drop(read); }}, _ = shutdown.changed() => return Ok(()), }}\n    }}\n}}"
            ),
            &[7],
        );
    }
}

#[test]
fn given_cancel_safe_and_pinned_branches_in_looping_select_when_checked_then_reports_only_acquire()
{
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "async fn forward(mut receiver: tokio::sync::mpsc::Receiver<u32>, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    loop { tokio::select! { value = receiver.recv() => { drop(value); }, _ = shutdown.changed() => break, } }\n}\nasync fn pinned(gate: tokio::sync::Mutex<()>, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    let queued = gate.lock();\n    tokio::pin!(queued);\n    loop { tokio::select! { guard = &mut queued => { drop(guard); }, _ = shutdown.changed() => break, } }\n}\nasync fn limited(permits: tokio::sync::Semaphore, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    loop { tokio::select! { permit = permits.acquire() => { drop(permit); }, _ = shutdown.changed() => break, } }\n}",
        &[10],
    );
}
