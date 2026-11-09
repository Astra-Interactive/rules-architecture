async fn run(receiver: tokio::sync::broadcast::Receiver<u32>) {
    while let Ok(event) = receiver.recv().await {}
}

