use std::sync::Arc;

pub(crate) struct TimerConfig;

pub(crate) enum TimerEndCause {
    Elapsed,
}

pub(crate) trait TimerListener: Send + Sync {
    fn on_started(&self, config: &TimerConfig);
    fn on_ended(&self, cause: TimerEndCause);
}

pub(crate) trait TimerRegistry {
    fn add_listener(&self, listener: Arc<dyn TimerListener>);
}
