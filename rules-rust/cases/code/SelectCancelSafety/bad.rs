async fn run(gate: tokio::sync::Mutex<()>) {
    loop { tokio::select! { _guard = gate.lock() => {}, else => {} } }
}

