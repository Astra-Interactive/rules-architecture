use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("could not read the workspace metadata")]
    Metadata {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("{path}{}: {message}", .line.map(|line| format!(":{line}")).unwrap_or_default())]
    Configuration {
        path: String,
        line: Option<u32>,
        message: String,
    },
    #[error("could not access {path}")]
    Io {
        path: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}
