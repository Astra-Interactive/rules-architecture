#[expect(clippy::disallowed_methods, reason = "wrong boundary")]
fn run() { let thread = std::thread::spawn(|| {}); }

