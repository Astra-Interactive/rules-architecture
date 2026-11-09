use crate::clock::api::Clock;
use crate::invoice::api::InvoiceRepository;
use crate::mail::api::MailSender;
use crate::reminder::model::ReminderSchedule;
use std::sync::Arc;

pub(crate) struct InvoiceReminder {
    invoice_repository: Arc<dyn InvoiceRepository>,
    schedule: ReminderSchedule,
    mail_sender: Arc<dyn MailSender>,
    clock: Arc<dyn Clock>,
}

impl InvoiceReminder {
    pub(crate) fn new(
        invoice_repository: Arc<dyn InvoiceRepository>,
        schedule: ReminderSchedule,
        mail_sender: Arc<dyn MailSender>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            invoice_repository,
            schedule,
            mail_sender,
            clock,
        }
    }
}
