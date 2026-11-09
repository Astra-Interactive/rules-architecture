use crate::shape::model::ProductId;

pub(crate) struct LineItem {
    pub(crate) product_id: ProductId,
    pub(crate) quantity: u32,
}
