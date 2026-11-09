use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::expression::rule::AdapterChainVisitor;

pub(crate) struct TrivialClosurePath;

impl Rule for TrivialClosurePath {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| &sources.files).flat_map(|file| {
            AdapterChainVisitor::collect(&file.syntax).into_iter().map(|span| build_node_finding("TrivialClosurePath", &file.path, span, file.path.clone(), "three or more path adapters obscure the operation; retain a closure when reaching a path needs more than two steps".to_owned()))
        }).collect()
    }
}
