mod event { struct UserEvent; impl tokio::Listener for UserEvent {} }
mod model { struct Observer; impl tokio::Listener for Observer {} }
