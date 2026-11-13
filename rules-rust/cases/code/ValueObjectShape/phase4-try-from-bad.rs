mod model {
    struct Money(pub i64);
    impl TryFrom<i64> for Money {
        type Error = ();
        fn try_from(cents: i64) -> Result<Self, ()> { if cents < 0 { return Err(()); } Ok(Self(cents)) }
    }
    struct Level { pub(crate) value: u8 }
    impl TryFrom<u8> for Level {
        type Error = ();
        fn try_from(value: u8) -> Result<Self, ()> { Ok(Self { value }) }
    }
}
