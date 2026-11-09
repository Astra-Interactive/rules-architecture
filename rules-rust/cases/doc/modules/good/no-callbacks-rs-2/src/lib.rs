pub(crate) mod timer {
    pub(crate) mod api {
        pub(crate) use timer_publisher::TimerPublisher;
        mod timer_publisher;
    }
    pub(crate) mod memory {
        pub(crate) use in_memory_timer::InMemoryTimer;
        mod in_memory_timer;
    }
    pub(crate) mod model {
        pub(crate) use timer_event::TimerEvent;
        mod timer_event;
    }
}
