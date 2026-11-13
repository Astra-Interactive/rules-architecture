// invoice/util/money_parse.rs
use crate::invoice::model::{Money, MoneyError};

pub(crate) fn parse_money(raw: &str) -> Result<Money, MoneyError> {
    let Ok(cents) = raw.trim().parse::<i64>() else {
        return Err(MoneyError::NotANumber {
            raw: raw.to_owned(),
        });
    };
    Money::try_from(cents)
}
