mod model {
    struct Money(i64);
    impl Money {
        fn new(cents: i64) -> Self { Self::try_from(cents).unwrap_or_else(|_| panic!("negative")) }
        fn cents(&self) -> i64 { self.0 }
        const ZERO: Money = Money(0);
    }
    impl TryFrom<i64> for Money {
        type Error = ();
        fn try_from(cents: i64) -> Result<Self, ()> { if cents < 0 { return Err(()); } Ok(Self(cents)) }
    }
    struct Level(u8);
    impl Level {
        fn new(value: u8) -> Self { assert!(value < 10); Self(value) }
        fn value(&self) -> u8 { self.0 }
    }
    impl std::convert::TryFrom<u8> for Level {
        type Error = ();
        fn try_from(value: u8) -> Result<Self, ()> { if value >= 10 { return Err(()); } Ok(Self(value)) }
    }
}
