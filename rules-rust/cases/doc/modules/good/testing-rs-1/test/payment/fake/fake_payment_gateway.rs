use crate::order::model::CustomerId;
use crate::payment::api::PaymentGateway;
use async_trait::async_trait;

pub(crate) struct FakePaymentGateway {
    is_declining: bool,
}

impl FakePaymentGateway {
    pub(crate) fn new() -> Self {
        Self {
            is_declining: false,
        }
    }
}

#[async_trait]
impl PaymentGateway for FakePaymentGateway {
    async fn charge(&self, _customer_id: CustomerId) -> bool {
        !self.is_declining
    }
}
