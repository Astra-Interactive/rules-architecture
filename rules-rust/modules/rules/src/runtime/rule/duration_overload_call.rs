use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TypedRuntimeVisitor;

pub(crate) struct DurationOverloadCall;

impl Rule for DurationOverloadCall {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TypedRuntimeVisitor::check_context(context, "DurationOverloadCall")
    }
}
