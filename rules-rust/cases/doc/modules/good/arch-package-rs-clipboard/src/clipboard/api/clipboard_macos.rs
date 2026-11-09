pub(crate) struct Clipboard {
    text: String,
}

impl Clipboard {
    pub(crate) fn read_text(&self) -> &str {
        &self.text
    }
}
