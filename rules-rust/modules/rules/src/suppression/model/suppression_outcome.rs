use crate::check::model::AcceptedException;
use crate::check::model::Finding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SuppressionOutcome {
    pub(crate) violations: Vec<Finding>,
    pub(crate) accepted: Vec<AcceptedException>,
}
