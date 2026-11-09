fn run(gate: parking_lot::Mutex<Vec<u32>>) {
    let mut guard = gate.lock();
    guard.push(1);
}

