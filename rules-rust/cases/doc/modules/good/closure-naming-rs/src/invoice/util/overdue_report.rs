use crate::invoice::model::Invoice;
use std::time::SystemTime;

pub(crate) fn build_overdue_report(
    invoices: &[Invoice],
    lines: &[String],
    now: SystemTime,
) -> Vec<String> {
    let overdue = invoices
        .iter()
        .filter(|invoice| invoice.due_at < now && !invoice.is_paid);
    let rows = lines
        .iter()
        .enumerate()
        .map(|(index, line)| format!("{index}: {line}"));
    let mut report: Vec<String> = rows.collect();
    report.push(format!("overdue: {}", overdue.count()));
    report
}
