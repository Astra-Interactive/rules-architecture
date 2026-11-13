mod di { pub struct RootModule; impl RootModule { pub fn new() -> Self { Self } } }
fn main() { let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async { let root = di::RootModule::new(); root.on_enable(); tokio::signal::ctrl_c().await; root }); root.on_disable(); }
