use rt::{Gate, start};
pub fn launch() {
    start(async {});
}
pub fn on_disable(gate: Gate<Vec<u32>>) {
    let _guard = gate.blocking_lock();
}

