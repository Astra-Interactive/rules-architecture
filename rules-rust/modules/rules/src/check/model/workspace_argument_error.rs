#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum WorkspaceArgumentError {
    #[error("usage: cargo klibs-rules [--manifest-path <path to Cargo.toml>]")]
    Usage,
}
