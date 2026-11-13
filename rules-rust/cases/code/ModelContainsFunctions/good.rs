mod model {
    struct Value { field: u64 }
    impl Value { fn new(field: u64) -> Self { assert!(field > 0); Self { field } } fn field(&self) -> u64 { self.field } const LIMIT: u64 = 1; }
    impl std::fmt::Display for Value {}
}
