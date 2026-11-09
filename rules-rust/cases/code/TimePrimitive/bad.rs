use std::time::Instant as Tick;
struct Stored { at: Tick }
fn now() -> Tick { loop {} }
const DELAY_MS: u64 = 40;

