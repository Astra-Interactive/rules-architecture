use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TestRuntimeVisitor;

pub(crate) struct ModelRuleTest;

impl Rule for ModelRuleTest {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TestRuntimeVisitor::check_context(context, "ModelRuleTest")
    }
}
