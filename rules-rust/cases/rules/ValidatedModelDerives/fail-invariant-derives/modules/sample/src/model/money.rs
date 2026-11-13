#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub(crate) struct Money(i64);

impl TryFrom<i64> for Money {
    type Error = ();

    fn try_from(cents: i64) -> Result<Self, Self::Error> {
        if cents < 0 {
            return Err(());
        }
        Ok(Self(cents))
    }
}
