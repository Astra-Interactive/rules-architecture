struct Duration;
impl Duration { fn from_millis(_: u128) -> Self { Self } fn as_millis(&self) -> u128 { 0 } }
fn run(delay: Duration) { let copy = Duration::from_millis(delay.as_millis()); }

