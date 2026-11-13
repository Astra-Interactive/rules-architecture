fn spawn<T>(_: T) {}
fn run() { spawn(1); let _ = spawn(2); }

