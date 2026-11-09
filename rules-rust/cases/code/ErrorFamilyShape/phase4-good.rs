type Source = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug, thiserror::Error)]
enum OrderError { #[error("failed")] Failed { #[source] source: Source } }
