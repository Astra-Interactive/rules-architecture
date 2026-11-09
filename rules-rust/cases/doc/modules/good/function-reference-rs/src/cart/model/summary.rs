use crate::cart::model::{ItemContent, UserId};
use crate::order::model::Order;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Summary {
    pub(crate) user_ids: Vec<UserId>,
    pub(crate) contents: Vec<ItemContent>,
    pub(crate) name_count: usize,
    pub(crate) labels: Vec<String>,
    pub(crate) order: Order,
}
