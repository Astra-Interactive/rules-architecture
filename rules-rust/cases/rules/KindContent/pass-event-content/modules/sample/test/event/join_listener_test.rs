use crate::event::JoinListener;

struct JoinListenerTest {
    join_listener: JoinListener,
}

impl JoinListenerTest {
    fn new() -> Self {
        Self {
            join_listener: JoinListener::new(String::from("welcome")),
        }
    }

    fn read_welcome(&self) -> &str {
        self.join_listener.on_join()
    }
}
