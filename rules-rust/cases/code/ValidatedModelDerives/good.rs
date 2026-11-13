mod model {
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)] #[serde(try_from = "i64")] struct Money(i64);
    impl TryFrom<i64> for Money { type Error = (); fn try_from(cents: i64) -> Result<Self, ()> { Ok(Self(cents)) } }
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Default)] struct Settings { limit: u64 }
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Default)] struct MoneyParts { pub(crate) cents: i64 }
}
