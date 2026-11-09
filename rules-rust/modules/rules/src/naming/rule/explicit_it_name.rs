use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::naming::rule::NameVisitor;

pub(crate) struct ExplicitItName;

impl Rule for ExplicitItName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        NameVisitor::check_context(context, "ExplicitItName")
    }
}
