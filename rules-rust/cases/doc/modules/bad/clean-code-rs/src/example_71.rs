use std::marker::PhantomData;

struct InvoiceReminderModule;

struct InvoiceReminderConfiguration;

struct ConfigurationFlagReader<Configuration>(PhantomData<Configuration>);

struct FeatureGate<Owner, Reader> {
    owner: PhantomData<Owner>,
    reader: PhantomData<Reader>,
}

type ReminderGate =
    FeatureGate<InvoiceReminderModule, ConfigurationFlagReader<InvoiceReminderConfiguration>>;
