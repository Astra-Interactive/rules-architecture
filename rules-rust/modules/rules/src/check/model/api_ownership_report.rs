pub struct ApiOwnershipReport {
    pub(crate) text: String,
}

impl ApiOwnershipReport {
    pub fn as_text(&self) -> &str {
        &self.text
    }
}
