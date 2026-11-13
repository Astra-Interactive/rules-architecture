use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TaskVisitor;
use crate::runtime::rule::TypedRuntimeVisitor;

pub(crate) struct ThreadPrimitive;

impl Rule for ThreadPrimitive {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = TaskVisitor::check_context(context, "ThreadPrimitive");
        findings.extend(TypedRuntimeVisitor::check_context(
            context,
            "ThreadPrimitive",
        ));
        findings
    }
}
