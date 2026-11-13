mod local { pub fn sleep() {} }
async fn run() { local::sleep(); let future = tokio::time::sleep(std::time::Duration::ZERO); }

