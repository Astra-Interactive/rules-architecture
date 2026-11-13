pub(crate) struct ViewCache {
    entry_count: u32,
}

impl ViewCache {
    pub(crate) fn new() -> Self {
        Self { entry_count: 0 }
    }

    pub(crate) fn entry_count(&self) -> u32 {
        self.entry_count
    }
}
