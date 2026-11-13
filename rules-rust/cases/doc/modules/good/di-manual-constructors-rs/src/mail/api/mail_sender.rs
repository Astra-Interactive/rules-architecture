pub(crate) trait MailSender: Send + Sync {
    fn send_reminder(&self, unpaid: usize) -> bool;
}
