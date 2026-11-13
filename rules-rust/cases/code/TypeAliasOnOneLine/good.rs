type KeepInventoryGate = FeatureGate<KeepInventoryModule, KeepInventoryFlagReader>;

pub(crate) type Gate = FeatureGate<KeepInventoryModule, Reader>;

type Result<T> = std::result::Result<T, Error>;

type ReminderFlagReader = ConfigurationFlagReader<InvoiceReminderConfiguration>;
type ReminderGate = FeatureGate<InvoiceReminderModule, ReminderFlagReader>;

/// First documentation line.
/// Second documentation line.
#[cfg(feature = "reminder")]
type DocumentedGate = FeatureGate<InvoiceReminderModule, ReminderFlagReader>;

struct Money;

impl Add for Money {
    type Output = Money;

    fn add(self, other: Money) -> Money {
        other
    }
}

impl Lending for Feed {
    type Item<'a> = &'a str;
}

trait Source {
    type Item: Clone
        + Send
        + Sync;
    type Id;
}

trait Defaulted {
    type Item: Clone = Vec<u8>;
}

impl LendingIterator for Feed {
    type Item<'a>
        = &'a str
    where
        Self: 'a;
}

type Alias<T>
= Vec<T>
where
    T: Clone;

m! {
    type A = Vec<u8>
}
