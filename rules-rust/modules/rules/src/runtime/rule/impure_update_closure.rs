use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::runtime::rule::TypedRuntimeVisitor;

pub(crate) struct ImpureUpdateClosure;

impl Rule for ImpureUpdateClosure {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        TypedRuntimeVisitor::check_context(context, "ImpureUpdateClosure")
    }
}
