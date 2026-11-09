mod event { struct UserEvent; impl std::fmt::Display for UserEvent {} }
mod model { struct Observer; impl tokio::Listener for Observer {} }
