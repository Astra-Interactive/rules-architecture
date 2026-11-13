use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub(crate) enum GateError {
    #[error("usage: cargo xtask gate-sync --docs <dir> [--check]")]
    Usage,
    #[error("could not read {path}")]
    Read {
        path: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("could not write {path}")]
    Write {
        path: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("examples.toml is malformed: {message}")]
    Examples { message: String },
}
