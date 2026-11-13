use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::runtime::rule::RuntimeVisitor;

pub(crate) struct BlockOnOutsideBoundary;

impl Rule for BlockOnOutsideBoundary {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        RuntimeVisitor::check_context(context, "BlockOnOutsideBoundary")
    }
}
