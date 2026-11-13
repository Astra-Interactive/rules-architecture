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
