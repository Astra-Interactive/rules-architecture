use crate::reminder::model::{
    ConfigurationFlagReader, FeatureGate, InvoiceReminderConfiguration, InvoiceReminderModule,
};

type ReminderFlagReader = ConfigurationFlagReader<InvoiceReminderConfiguration>;
type ReminderGate = FeatureGate<InvoiceReminderModule, ReminderFlagReader>;
