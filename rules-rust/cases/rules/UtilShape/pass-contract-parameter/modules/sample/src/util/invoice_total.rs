use crate::model::Invoice;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

pub(crate) fn compute_total(invoice: &Invoice, rate: u32) -> u64 {
    0
}

pub(crate) fn read_moment(moment: SystemTime) {}

pub(crate) fn run_step<F: Fn(u32) -> u32>(step: F) {}

pub(crate) fn count_all(items: &[Invoice], cache: Arc<Mutex<u32>>) {}

pub(crate) fn apply_step(step: impl Fn(u32) -> u32) {}
