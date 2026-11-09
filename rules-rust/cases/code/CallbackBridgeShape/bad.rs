struct Registration<F> { callback: F } fn consume<F: Fn()>(callback: F) -> Registration<F> { Registration { callback } }
fn run() { consume(|| { tokio::spawn(async {}); }); }

