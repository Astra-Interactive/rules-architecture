use crate::cart::model::{CartItem, ItemContent};

impl CartItem {
    pub(crate) fn to_item_content(&self) -> ItemContent {
        ItemContent {
            label: self.name.clone(),
        }
    }
}
