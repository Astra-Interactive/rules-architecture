mod model {
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)] struct Money(i64);
    impl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Default)] struct Level(u8);
    impl TryFrom<u8> for Level { type Error = (); fn try_from(value: u8) -> Result<Self, ()> { Ok(Self(value)) } }
}
