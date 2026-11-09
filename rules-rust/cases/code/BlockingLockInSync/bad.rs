type Gate = tokio::sync::Mutex<u32>;
struct Owner { gate: Gate }
impl Owner { fn on_disable(&self) { self.gate.blocking_lock(); } }

