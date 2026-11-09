use crate::timer::model::{TimerConfig, TimerEndCause};

pub(crate) enum TimerEvent {
    Started { config: TimerConfig },
    Ended { cause: TimerEndCause },
}
