use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TypedRuntimeVisitor;

pub(crate) struct LockChoice;

impl Rule for LockChoice {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TypedRuntimeVisitor::check_context(context, "LockChoice")
    }
}
