mod model {
    struct Value { field: u64 }
    impl Value { fn new(field: u64) -> Self { Self { field } } fn read(&self) -> u64 { self.field } }
    impl Value { const LIMIT: u64 = 1; }
}
