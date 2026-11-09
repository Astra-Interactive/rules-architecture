#[derive(Debug, thiserror::Error)]
enum OrderError { #[error("failed")] Failed { #[source] source: std::io::Error } }
