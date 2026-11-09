use std::time::SystemTime;

pub(crate) struct Invoice {
    pub(crate) due_at: SystemTime,
    pub(crate) is_paid: bool,
}

pub(crate) fn build_overdue_report(invoices: &[Invoice], lines: &[String], now: SystemTime) -> Vec<String> {
    let overdue = invoices.iter().filter(|i| i.due_at < now && !i.is_paid);
    let rows = lines
        .iter()
        .enumerate()
        .map(|(n, line)| format!("{n}: {line}"));
    let mut report: Vec<String> = rows.collect();
    report.push(format!("overdue: {}", overdue.count()));
    report
}
