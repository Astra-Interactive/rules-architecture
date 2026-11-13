use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::expression::rule::ExpressionVisitor;

pub(crate) struct OperatorBeforeTrailingCall;

impl Rule for OperatorBeforeTrailingCall {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        ExpressionVisitor::check_context(context, "OperatorBeforeTrailingCall")
    }
}
