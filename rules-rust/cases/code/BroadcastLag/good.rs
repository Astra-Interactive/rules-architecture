async fn run(receiver: tokio::sync::mpsc::Receiver<u32>) {
    while let Ok(event) = receiver.recv().await {}
}

