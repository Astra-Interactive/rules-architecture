#[cfg(test)]
mod test { fn work() {}
    use tokio::runtime::{Runtime as AsyncRuntime, Builder as AsyncBuilder};
    fn run() {
        let runtime = AsyncRuntime::new();
        let runtime = AsyncBuilder::new_current_thread();
        runtime.block_on(work());
        futures::executor::block_on(work());
    }
}
