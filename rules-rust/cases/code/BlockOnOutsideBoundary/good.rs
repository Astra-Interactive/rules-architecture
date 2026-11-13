mod file { fn work() {}
    fn run(handle: tokio::runtime::Handle) { handle.block_on(work()); }
}
struct Builder;
impl Builder { fn new_current_thread() {} }
fn run() { Builder::new_current_thread(); }
