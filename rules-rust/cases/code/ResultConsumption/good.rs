fn run(sender: tokio::sync::oneshot::Sender<u32>) {
    #[expect(clippy::let_underscore_must_use, reason = "receiver is gone")]
    let _ = sender.send(1);
}

