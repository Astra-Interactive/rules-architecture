// invoice/model/payment_plan.rs, an aggregate root; its Parts struct is in payment_plan_parts.rs
use crate::invoice::model::{Money, PaymentPlanError, PaymentPlanParts};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "PaymentPlanParts")]
pub(crate) struct PaymentPlan {
    total: Money,
    installments: Vec<Money>,
}

impl TryFrom<PaymentPlanParts> for PaymentPlan {
    type Error = PaymentPlanError;

    fn try_from(parts: PaymentPlanParts) -> Result<Self, Self::Error> {
        let paid_in: i64 = parts.installments.iter().map(Money::cents).sum();
        if paid_in != parts.total.cents() {
            return Err(PaymentPlanError::Mismatch { total: parts.total });
        }
        Ok(Self {
            total: parts.total,
            installments: parts.installments,
        })
    }
}
