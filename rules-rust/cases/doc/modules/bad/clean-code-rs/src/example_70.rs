use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug)]
struct InvoiceId(u64);

struct Invoice {
    id: InvoiceId,
}

#[derive(Debug, thiserror::Error)]
enum InvoiceError {
    #[error("could not serialize invoice {}", .id.0)]
    Serialize {
        id: InvoiceId,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}

fn to_json<Document>(_document: &Document) -> Result<String, fmt::Error> {
    Ok(String::new())
}

fn serialize_reminder(reminder_invoice: Invoice, prefix: &str) -> Result<String, InvoiceError> {
    let serialized_invoice =
        to_json(&reminder_invoice).map_err(|source| InvoiceError::Serialize {
            id: reminder_invoice.id,
            source: Box::new(source),
        })?;
    Ok(format!("{prefix}{serialized_invoice}"))
}
