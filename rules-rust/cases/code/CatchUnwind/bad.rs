fn local() {}
fn run() { let result = std::panic::catch_unwind(|| local()); }

