async fn run(receiver: tokio::sync::watch::Receiver<bool>) {
    tokio::time::timeout(std::time::Duration::ZERO, receiver.wait_for(|ready| *ready));
    tokio::time::timeout(std::time::Duration::ZERO, async { receiver.changed().await });
}

