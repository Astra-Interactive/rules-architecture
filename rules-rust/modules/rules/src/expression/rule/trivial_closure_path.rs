use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::expression::rule::AdapterChainVisitor;

pub(crate) struct TrivialClosurePath;

impl Rule for TrivialClosurePath {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().enumerate().flat_map(|(ordinal, sources)| sources.files.iter().map(move |file| (ordinal, file))).flat_map(|(ordinal, file)| {
            AdapterChainVisitor::collect(context, ordinal, file).into_iter().map(|span| build_node_finding("TrivialClosurePath", &file.path, span, file.path.clone(), "Use function paths for a two-step parameter adaptation; retain a closure when reaching a path needs more than two steps".to_owned()))
        }).collect()
    }
}
