use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TaskVisitor;

pub(crate) struct DetachedTask;

impl Rule for DetachedTask {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TaskVisitor::check_context(context, "DetachedTask")
    }
}
