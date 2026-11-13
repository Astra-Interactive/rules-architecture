use crate::order::model::{Money, Order};

pub(crate) struct OrderPricing {
    tax_rate: f64,
}

impl OrderPricing {
    pub(crate) fn new(tax_rate: f64) -> Self {
        Self { tax_rate }
    }

    fn apply_tax(&self, subtotal: Money) -> Money {
        subtotal * (1.0 + self.tax_rate)
    }

    pub(crate) fn compute_price(&self, order: &Order) -> Money {
        if order.items.len() > Self::MAX_ITEMS {
            return Money::ZERO;
        }
        self.apply_tax(order.subtotal)
    }

    const MAX_ITEMS: usize = 100;
}
