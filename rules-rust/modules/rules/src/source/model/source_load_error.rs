use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SourceLoadError {
    #[error("{path} cannot be read: {source}")]
    Read {
        path: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("{path} does not parse: {source}")]
    Parse {
        path: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}
