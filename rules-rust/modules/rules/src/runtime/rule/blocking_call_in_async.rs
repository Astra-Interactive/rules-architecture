use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TaskVisitor;
use crate::runtime::rule::TypedRuntimeVisitor;

pub(crate) struct BlockingCallInAsync;

impl Rule for BlockingCallInAsync {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = TaskVisitor::check_context(context, "BlockingCallInAsync");
        findings.extend(TypedRuntimeVisitor::check_context(
            context,
            "BlockingCallInAsync",
        ));
        findings
    }
}
