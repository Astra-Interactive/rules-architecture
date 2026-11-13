use crate::invoice::model::Money;

#[derive(Debug, thiserror::Error)]
pub(crate) enum PaymentPlanError {
    #[error("the installments do not add up to {total:?}")]
    Mismatch { total: Money },
}
