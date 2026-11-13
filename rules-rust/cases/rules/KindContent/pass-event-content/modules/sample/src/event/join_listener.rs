pub(crate) struct JoinListener {
    join_message: JoinMessage,
}

impl JoinListener {
    pub(crate) fn new(text: String) -> Self {
        Self {
            join_message: JoinMessage { text },
        }
    }

    pub(crate) fn on_join(&self) -> &str {
        &self.join_message.text
    }
}

struct JoinMessage {
    text: String,
}
