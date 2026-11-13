use crate::runtime::rule::SelectCancelSafety;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_write_all_on_a_generic_tokio_writer_in_looping_select_when_checked_then_reports_the_branch()
 {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "use tokio::io::AsyncWriteExt;\nasync fn pump<S: AsyncWriteExt + Unpin>(stream: &mut S, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    loop { tokio::select! { written = stream.write_all(b\"x\") => { drop(written); }, _ = shutdown.changed() => return, } }\n}\nasync fn pump_where<S>(stream: &mut S, mut shutdown: tokio::sync::watch::Receiver<bool>) where S: tokio::io::AsyncWriteExt + Unpin {\n    loop { tokio::select! { written = stream.write_all(b\"x\") => { drop(written); }, _ = shutdown.changed() => return, } }\n}\ntrait Sink { async fn write_all(&mut self, _: &[u8]) {} }\nasync fn own<S: Sink>(stream: &mut S, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    loop { tokio::select! { written = stream.write_all(b\"x\") => { drop(written); }, _ = shutdown.changed() => return, } }\n}",
        &[3, 6],
    );
}

#[test]
fn given_workspace_async_fn_awaiting_a_cancel_unsafe_read_in_looping_select_when_checked_then_reports_the_branch()
 {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "use tokio::io::AsyncReadExt;\nasync fn read_frame(stream: &mut tokio::net::TcpStream, buffer: &mut [u8; 4]) -> std::io::Result<()> { stream.read_exact(buffer).await.map(|_| ()) }\nasync fn next_value(inbox: &mut tokio::sync::mpsc::Receiver<u32>) -> Option<u32> { inbox.recv().await }\nasync fn pump(stream: &mut tokio::net::TcpStream, mut inbox: tokio::sync::mpsc::Receiver<u32>, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    let mut buffer = [0u8; 4];\n    loop {\n        tokio::select! { read = read_frame(stream, &mut buffer) => { drop(read); }, _ = shutdown.changed() => return, }\n    }\n    loop {\n        tokio::select! { value = next_value(&mut inbox) => { drop(value); }, _ = shutdown.changed() => return, }\n    }\n}",
        &[7],
    );
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "use tokio::io::AsyncReadExt;\nstruct Reader { stream: tokio::net::TcpStream, buffer: [u8; 4] }\nimpl Reader {\n    async fn fill(&mut self) { drop(self.stream.read_exact(&mut self.buffer).await); }\n    async fn pump(&mut self, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n        loop { tokio::select! { filled = self.fill() => { drop(filled); }, _ = shutdown.changed() => return, } }\n    }\n}",
        &[6],
    );
}

#[test]
fn given_other_cancel_unsafe_reads_in_looping_select_when_checked_then_reports_each_branch() {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "use tokio::io::AsyncBufReadExt;\nuse tokio::io::AsyncReadExt;\nasync fn pump(reader: &mut tokio::io::BufReader<tokio::net::TcpStream>, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n    let mut line = String::new();\n    let mut buffer = [0u8; 4];\n    loop {\n        tokio::select! { read = reader.read_line(&mut line) => { drop(read); }, _ = shutdown.changed() => return, }\n    }\n    loop {\n        tokio::select! { value = reader.read_u32() => { drop(value); }, _ = shutdown.changed() => return, }\n    }\n    loop {\n        tokio::select! { read = reader.read(&mut buffer) => { drop(read); }, _ = shutdown.changed() => return, }\n    }\n}",
        &[7, 10],
    );
}

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
fn given_callee_from_another_module_awaiting_read_exact_when_looping_select_then_reports_branch() {
    assert_code_findings(
        &SelectCancelSafety,
        "SelectCancelSafety",
        "mod net {\n    use tokio::{io::AsyncReadExt, net::TcpStream};\n    pub(crate) async fn read_frame(s: &mut TcpStream, b: &mut [u8]) { s.read_exact(b).await.unwrap(); }\n}\nmod app {\n    pub(crate) async fn pump(mut s: tokio::net::TcpStream, mut shutdown: tokio::sync::watch::Receiver<bool>) {\n        let mut b = [0u8; 4];\n        loop {\n            tokio::select! { read = crate::net::read_frame(&mut s, &mut b) => { drop(read); }, _ = shutdown.changed() => return, }\n        }\n    }\n}",
        &[9],
    );
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
