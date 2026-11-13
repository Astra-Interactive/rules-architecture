// invoice/model/money.rs
use crate::invoice::model::MoneyError;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "i64")]
pub(crate) struct Money(i64);

impl Money {
    pub(crate) fn new(cents: i64) -> Self {
        Self::try_from(cents).unwrap_or_else(|error| panic!("{error}"))
    }

    pub(crate) fn cents(&self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for Money {
    type Error = MoneyError;

    fn try_from(cents: i64) -> Result<Self, Self::Error> {
        if cents < 0 {
            return Err(MoneyError::Negative { cents });
        }
        Ok(Self(cents))
    }
}
