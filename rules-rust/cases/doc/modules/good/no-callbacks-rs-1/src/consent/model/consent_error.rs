#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConsentError {
    Rejected { reason: String },
    Abandoned,
}
