async fn run(gate: tokio::sync::Mutex<()>) {
    let future = gate.lock();
    tokio::pin!(future);
    loop { tokio::select! { _guard = &mut future => {}, else => {} } }
}

