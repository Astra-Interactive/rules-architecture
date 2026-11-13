use crate::invoice::model::Invoice;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Write as _;
use tokio::sync::watch;

pub(crate) fn write_invoice_report<T>(
    report: &mut String,
    totals: &HashMap<u32, Invoice>,
    latest: &watch::Receiver<T>,
) -> std::fmt::Result
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    for (number, invoice) in totals {
        writeln!(report, "{number}: {}", invoice.total_cents)?;
    }
    writeln!(report, "latest: {:?}", *latest.borrow())
}
