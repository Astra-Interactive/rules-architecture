use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;

pub(crate) trait Rule: Send + Sync {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding>;
}
