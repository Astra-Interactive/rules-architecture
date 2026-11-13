mod model {
    struct Money(i64);
    impl Money { fn cents(&self) -> i64 { self.0 } }
    impl TryFrom<i64> for Money {
        type Error = ();
        fn try_from(cents: i64) -> Result<Self, ()> { if cents < 0 { return Err(()); } Ok(Self(cents)) }
    }
    struct Plain(pub i64);
    struct MoneyParts { pub(crate) cents: i64 }
}
