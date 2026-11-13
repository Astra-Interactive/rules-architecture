use crate::invoice::model::Money;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub(crate) struct PaymentPlanParts {
    pub(crate) total: Money,
    pub(crate) installments: Vec<Money>,
}
