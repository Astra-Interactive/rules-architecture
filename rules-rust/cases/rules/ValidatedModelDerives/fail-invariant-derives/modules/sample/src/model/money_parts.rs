#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, Default)]
pub(crate) struct MoneyParts {
    pub(crate) cents: i64,
}
