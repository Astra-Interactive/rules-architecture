mod model {
    pub(crate) struct Value { pub field: u64 }
    impl Value { pub fn read(&self) -> u64 { self.field } }
    pub struct Hidden { pub field: u64 }
}
