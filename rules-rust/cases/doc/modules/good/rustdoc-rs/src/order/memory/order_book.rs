use crate::order::model::Order;

pub struct OrderBook {
    items: Vec<Order>,
}

impl OrderBook {
    pub fn new(items: Vec<Order>) -> Self {
        Self { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// # Safety
    ///
    /// `index` is less than `self.len()`; the bounds check is skipped.
    pub unsafe fn get_unchecked(&self, index: usize) -> &Order {
        // SAFETY: the caller keeps `index < self.len()`, as the `# Safety` section above requires.
        unsafe { self.items.get_unchecked(index) }
    }
}
