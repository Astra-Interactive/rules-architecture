mod model {
    struct Money(i64);
    impl Money {
        fn new(cents: i64) -> Self { Self(cents) }
        fn cents(&self) -> i64 { self.0 }
        fn is_zero(&self) -> bool { self.0 == 0 }
    }
    impl TryFrom<i64> for Money {
        type Error = ();
        fn try_from(cents: i64) -> Result<Self, ()> { if cents < 0 { return Err(()); } Ok(Self(cents)) }
    }
    struct Plain(i64);
    impl Plain {
        fn new(cents: i64) -> Self { Self(cents) }
        fn cents(&self) -> i64 { self.0 }
    }
}
