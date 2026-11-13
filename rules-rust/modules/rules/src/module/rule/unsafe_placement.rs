use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::rule::UnsafeVisitor;

pub(crate) struct UnsafePlacement;

impl Rule for UnsafePlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                context
                    .crates
                    .iter()
                    .filter(move |sources| {
                        sources.crate_name == index.crate_name && sources.target == index.target
                    })
                    .flat_map(move |sources| {
                        sources
                            .files
                            .iter()
                            .flat_map(move |file| UnsafeVisitor::check_file(context, ordinal, file))
                    })
            })
            .collect()
    }
}
