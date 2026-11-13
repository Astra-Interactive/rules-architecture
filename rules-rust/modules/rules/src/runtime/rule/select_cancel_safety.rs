use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TypedRuntimeVisitor;

pub(crate) struct SelectCancelSafety;

impl Rule for SelectCancelSafety {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TypedRuntimeVisitor::check_context(context, "SelectCancelSafety")
    }
}
