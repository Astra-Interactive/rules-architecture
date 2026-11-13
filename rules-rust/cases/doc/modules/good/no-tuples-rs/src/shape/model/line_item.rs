use crate::shape::model::ProductId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct LineItem {
    pub(crate) product_id: ProductId,
    pub(crate) quantity: u32,
}
