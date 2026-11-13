use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::source::model::{CrateIndex, Resolution, SourceEdge};

pub(crate) struct ImportThroughFileModule;

impl ImportThroughFileModule {
    fn walks_file_module(index: &CrateIndex, edge: &SourceEdge) -> bool {
        let is_own_export = index.imports.iter().any(|binding| {
            binding.module == edge.source
                && binding.span == edge.span
                && binding.is_public
                && index.modules[edge.target].parent == Some(edge.source)
        });
        if is_own_export {
            return false;
        }
        (1..edge.path.len()).any(|length| {
            matches!(index.resolve_segments(edge.source, &edge.path[..length], 0), Resolution::Internal(targets) if targets.iter().any(|target| target.item.is_none() && index.modules[target.module].is_file_module))
        })
    }
}

impl Rule for ImportThroughFileModule {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().flat_map(|index| index.edges.iter().filter(|edge| Self::walks_file_module(index, edge)).map(|edge| build_node_finding("ImportThroughFileModule", &edge.file, edge.span, edge.path.join("::"), "Import and name an item through its kind block, without walking through its private file module".to_owned()))).collect()
    }
}
