struct SystemClock;
trait Clock { fn now(&mut self) -> std::time::SystemTime; }
