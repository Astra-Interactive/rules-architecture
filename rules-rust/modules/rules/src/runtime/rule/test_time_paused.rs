use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TestRuntimeVisitor;

pub(crate) struct TestTimePaused;

impl Rule for TestTimePaused {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TestRuntimeVisitor::check_context(context, "TestTimePaused")
    }
}
