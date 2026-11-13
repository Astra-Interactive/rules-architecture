pub mod model {
    pub use hidden::Value;
    mod hidden { pub struct Value { pub field: u64 } }
    pub(crate) struct Private { pub(crate) field: u64 }
    impl Private { pub(crate) fn read(&self) -> u64 { self.field } }
}
