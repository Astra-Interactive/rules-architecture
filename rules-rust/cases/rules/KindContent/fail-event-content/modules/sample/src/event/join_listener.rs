use crate::event::ViewCache;

pub(crate) struct JoinListener {
    view_cache: ViewCache,
}

impl JoinListener {
    pub(crate) fn new(view_cache: ViewCache) -> Self {
        Self { view_cache }
    }

    pub(crate) fn on_join(&self) -> u32 {
        self.view_cache.entry_count()
    }
}
