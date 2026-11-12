use crate::invoice::model::{Invoice, InvoiceError};
use crate::invoice::util::to_json;

pub(crate) fn serialize_reminder(
    reminder_invoice: Invoice,
    prefix: &str,
) -> Result<String, InvoiceError> {
    let serialization = to_json(&reminder_invoice);
    let serialized_invoice = serialization.map_err(|source| InvoiceError::Serialize {
        id: reminder_invoice.id,
        source: Box::new(source),
    })?;
    Ok(format!("{prefix}{serialized_invoice}"))
}
