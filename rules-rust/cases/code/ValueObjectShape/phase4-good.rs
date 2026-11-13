mod model { struct Money(i64); impl Money { fn new(cents: i64) -> Self { assert!(cents >= 0); Self(cents) } fn cents(&self) -> i64 { self.0 } } }
