type Shared = std::sync::Mutex<u64>;
static BAD: Shared = todo!();
