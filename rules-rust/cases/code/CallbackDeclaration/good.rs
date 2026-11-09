fn map(transform: impl Fn(String)) {}
impl External for Gateway { fn run(&self, on_result: impl Fn()) {} }
mod test { trait TimerListener {} fn run(on_event: fn()) {} }
mod fixture { trait CompletionCallback {} struct Sample { on_event: fn() } }
