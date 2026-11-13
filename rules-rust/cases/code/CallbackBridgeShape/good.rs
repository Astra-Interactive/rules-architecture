fn consume<F: Fn()>(_: F) {}
fn spawn() {}
fn run() { consume(|| spawn()); let handle = tokio::spawn(async {}); }

