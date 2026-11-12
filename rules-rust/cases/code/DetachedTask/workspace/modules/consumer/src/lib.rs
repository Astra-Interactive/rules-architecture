use rt::{Gate, start};
pub fn launch() {
    start(async {});
}
impl Owner { fn on_reload(&self, gate: Gate<Vec<u32>>) {
    let _guard = gate.blocking_lock();
} }
struct Owner;

