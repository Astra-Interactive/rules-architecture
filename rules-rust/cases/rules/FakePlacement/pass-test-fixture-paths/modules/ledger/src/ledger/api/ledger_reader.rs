pub trait LedgerReader: Send + Sync {
    fn total(&self) -> u32;
}
