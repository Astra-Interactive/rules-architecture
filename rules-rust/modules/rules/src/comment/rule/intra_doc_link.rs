use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::comment::rule::DocLinkVisitor;

pub(crate) struct IntraDocLink;

impl Rule for IntraDocLink {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| {
                let ordinal = context.indexes.iter().position(|index| {
                    index.crate_name == sources.crate_name && index.target == sources.target
                });
                sources.files.iter().flat_map(move |file| {
                    ordinal
                        .map(|ordinal| DocLinkVisitor::collect(context, ordinal, file))
                        .unwrap_or_default()
                })
            })
            .collect()
    }
}
