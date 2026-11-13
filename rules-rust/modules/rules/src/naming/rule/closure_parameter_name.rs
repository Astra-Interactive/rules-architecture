use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::naming::rule::NameVisitor;

pub(crate) struct ClosureParameterName;

impl Rule for ClosureParameterName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        NameVisitor::check_context(context, "ClosureParameterName")
    }
}
