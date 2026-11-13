use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TypedRuntimeVisitor;

pub(crate) struct JoinSetBehindLock;

impl Rule for JoinSetBehindLock {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TypedRuntimeVisitor::check_context(context, "JoinSetBehindLock")
    }
}
