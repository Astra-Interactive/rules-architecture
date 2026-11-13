use std::fmt;

pub(crate) fn to_json<Document>(_document: &Document) -> Result<String, fmt::Error> {
    Ok(String::new())
}
