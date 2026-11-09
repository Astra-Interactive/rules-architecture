use tokio::spawn as start;
async fn work() {}
fn run() {
    start(work());
    let _ = start(work());
    drop(start(work()));
    let owned = start(work());
}

