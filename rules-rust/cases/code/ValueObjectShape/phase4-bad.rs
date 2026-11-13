mod model {
 struct Money(pub i64);
 impl Money { fn new(cents: i64) -> Self { assert!(cents >= 0); Self(cents) } }
}
