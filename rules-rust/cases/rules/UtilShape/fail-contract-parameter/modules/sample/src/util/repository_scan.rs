use crate::api::Repository;
use std::sync::Arc;

pub(crate) fn load_all(repository: &dyn Repository) {}

pub(crate) fn store_all(repository: Arc<dyn Repository>) {}

pub(crate) fn drop_all(repository: Box<dyn Repository>) {}

pub(crate) fn scan_all(repository: impl Repository) {}

pub(crate) fn walk_all<R: Repository>(repository: R) {}

pub(crate) fn sweep_all<R>(repository: R)
where
    R: Repository,
{
}
