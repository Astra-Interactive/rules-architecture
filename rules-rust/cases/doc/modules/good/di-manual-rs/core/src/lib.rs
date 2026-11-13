pub mod clock {
    pub mod api {
        pub use clock::Clock;
        mod clock;
    }
    pub(crate) mod system {
        pub(crate) use system_clock::SystemClock;
        mod system_clock;
    }
}
pub mod di {
    pub use core_module::CoreModule;
    mod core_module;
}
pub mod lifecycle {
    pub mod api {
        pub use lifecycle::Lifecycle;
        mod lifecycle;
    }
}
