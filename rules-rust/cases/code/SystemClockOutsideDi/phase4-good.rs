mod system { pub struct SystemClock; }
trait Clock { fn now(&self) -> std::time::SystemTime; }
mod di { fn build() { let clock = crate::system::SystemClock; } }
