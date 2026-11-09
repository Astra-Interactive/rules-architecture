use crate::check::model::{ApiOwnershipReport, Report};

pub struct CheckedWorkspace {
    pub report: Report,
    pub api_ownership: ApiOwnershipReport,
}
