use crate::check::model::AcceptedException;
use crate::check::model::Finding;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub violations: Vec<Finding>,
    pub accepted: Vec<AcceptedException>,
    pub target_directory: PathBuf,
}
