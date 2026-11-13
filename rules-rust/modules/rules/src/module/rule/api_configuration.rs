use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::source::model::Resolution;

pub(crate) struct ApiConfiguration;

impl Rule for ApiConfiguration {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().flat_map(|index| index.imports.iter().filter_map(|binding| {
            if !binding.is_public || !matches!(index.resolve_segments(binding.module, &binding.path, 0), Resolution::External(_)) { return None; }
            Some(build_node_finding("ApiConfiguration", &binding.file, binding.span, binding.path.join("::"), "A dependency is private to its crate; do not re-export external crate items (arch-package.rs.md, Units and Cargo)".to_owned()))
        })).collect()
    }
}
