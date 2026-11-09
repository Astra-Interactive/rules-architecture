use crate::check::model::Location;
use crate::check::model::RuleId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedException {
    pub rule_id: RuleId,
    pub location: Location,
    pub subject: String,
    pub reason: String,
}
