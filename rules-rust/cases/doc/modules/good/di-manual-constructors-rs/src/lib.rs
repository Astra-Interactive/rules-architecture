pub(crate) mod clock {
    pub(crate) mod api {
        pub(crate) use clock::Clock;
        mod clock;
    }
}
pub(crate) mod invoice {
    pub(crate) mod api {
        pub(crate) use invoice_repository::InvoiceRepository;
        mod invoice_repository;
    }
}
pub(crate) mod mail {
    pub(crate) mod api {
        pub(crate) use mail_sender::MailSender;
        mod mail_sender;
    }
}
pub(crate) mod reminder {
    pub(crate) mod model {
        pub(crate) use reminder_schedule::ReminderSchedule;
        mod reminder_schedule;
    }
    pub(crate) mod service {
        pub(crate) use invoice_reminder::InvoiceReminder;
        mod invoice_reminder;
    }
}
