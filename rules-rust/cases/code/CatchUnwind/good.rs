fn run() { let result = std::panic::catch_unwind(|| std::fs::read("file")); }

