trait TimerListener { fn tick(&self); }
trait CompletionCallback {}
fn load(on_result: impl FnOnce(String)) {}
fn load_generic<F>(on_result: F) where F: FnMut(String) {}
struct Gateway<F: Fn()> { on_event: Option<Box<dyn Fn()>>, on_done: F }
