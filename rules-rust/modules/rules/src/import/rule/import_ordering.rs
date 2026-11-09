use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::import::rule::ImportOrderVisitor;
use crate::source::util::find_node_span;

pub(crate) struct ImportOrdering;

impl Rule for ImportOrdering {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| &sources.files).filter(|file| !file.is_generated).flat_map(|file| {
            ImportOrderVisitor::new(&file.comments, !file.module.path.is_empty()).collect_findings(&file.syntax).into_iter().map(|span| build_node_finding("ImportOrdering", &file.path, find_node_span(span), format_module_path(&file.module.crate_name, &file.module.path), "Keep use declarations first and file imports in one contiguous block (imports.rs.md; code-ordering.rs.md)".to_owned())).collect::<Vec<_>>()
        }).collect()
    }
}
