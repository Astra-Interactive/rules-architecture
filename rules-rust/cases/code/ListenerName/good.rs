mod event { struct UserListener; impl tokio::Listener for UserListener {} }
mod model { struct Value; }
mod util { impl std::fmt::Display for crate::model::Value {} }
