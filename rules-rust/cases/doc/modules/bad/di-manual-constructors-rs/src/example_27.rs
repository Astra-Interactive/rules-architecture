use std::sync::Arc;

pub(crate) trait InvoiceRepository: Send + Sync {}

pub(crate) trait MailSender: Send + Sync {}

pub(crate) trait Clock: Send + Sync {}

pub(crate) struct ReminderSchedule;

impl ReminderSchedule {
    pub(crate) fn new() -> Self {
        Self
    }
}

pub(crate) struct SmtpMailSender;

impl SmtpMailSender {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl MailSender for SmtpMailSender {}

pub(crate) struct SystemClock;

impl Clock for SystemClock {}

pub(crate) struct InvoiceReminder {
    invoice_repository: Arc<dyn InvoiceRepository>,
    schedule: ReminderSchedule,
    mail_sender: Arc<dyn MailSender>,
    clock: Arc<dyn Clock>,
}

impl InvoiceReminder {
    pub(crate) fn new(invoice_repository: Arc<dyn InvoiceRepository>) -> Self {
        Self {
            invoice_repository,
            schedule: ReminderSchedule::new(),
            mail_sender: Arc::new(SmtpMailSender::new()),
            clock: Arc::new(SystemClock),
        }
    }
}
