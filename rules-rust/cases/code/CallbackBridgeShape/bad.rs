fn consume<F: Fn()>(_: F) {}
fn run() { consume(|| { tokio::spawn(async {}); }); }

