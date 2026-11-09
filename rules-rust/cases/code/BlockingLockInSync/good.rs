struct Mutex;
impl Mutex { fn blocking_lock(&self) {} }
struct Owner { gate: Mutex }
impl Owner { fn on_disable(&self) { self.gate.blocking_lock(); } }

