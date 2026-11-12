type Gate = tokio::sync::Mutex<u32>;
struct Owner { gate: Gate }
impl Owner { fn on_reload(&self) { self.gate.blocking_lock(); } }

