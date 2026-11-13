async fn run(receiver: tokio::sync::watch::Receiver<bool>) {
    tokio::time::timeout(std::time::Duration::ZERO, receiver.wait_for(|_| true));
    tokio::time::timeout(std::time::Duration::ZERO, async { 1 });
}

