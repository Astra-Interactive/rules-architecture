use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::config::model::Role;
use crate::source::util::find_node_span;
use crate::suppression::rule::LintExpectationVisitor;

pub(crate) struct ExpectScope;

impl ExpectScope {
    fn allows_print(context: &RuleContext, crate_name: &str) -> bool {
        context
            .find_role_config(crate_name)
            .and_then(|config| config.role)
            .is_some_and(|role| matches!(role, Role::EntryPoint | Role::Tooling))
    }
}

impl Rule for ExpectScope {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| &sources.files).flat_map(|file| {
            let allows_print = Self::allows_print(context, &file.module.crate_name);
            LintExpectationVisitor::new(allows_print).collect_findings(&file.syntax).into_iter().map(|span| build_node_finding("ExpectScope", &file.path, find_node_span(span), format_module_path(&file.module.crate_name, &file.module.path), "Expect only a prescribed lint on the smallest item, never an inner attribute or module (notes.rs.md)".to_owned())).collect::<Vec<_>>()
        }).collect()
    }
}
