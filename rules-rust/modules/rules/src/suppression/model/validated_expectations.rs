use crate::check::model::Finding;
use crate::suppression::model::ExpectationSlot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedExpectations {
    pub(crate) slots: Vec<ExpectationSlot>,
    pub(crate) findings: Vec<Finding>,
}
