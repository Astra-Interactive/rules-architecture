use tokio::runtime::{Runtime as AsyncRuntime, Builder as AsyncBuilder};
#[tokio::main]
async fn boot() {}
fn run() {
    let runtime = AsyncRuntime::new();
    let runtime = AsyncBuilder::new_multi_thread().build().unwrap();
    runtime.block_on(work());
    futures::executor::block_on(work());
}
fn work() {}
