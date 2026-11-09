use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::source::model::Resolution;

pub(crate) struct ForbiddenImport;

impl Rule for ForbiddenImport {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().enumerate().flat_map(|(ordinal, index)| index.references.iter().filter_map(move |reference| {
            let Resolution::External(path) = &reference.resolution else { return None; };
            let identity = context.resolve_external_identity(ordinal, path);
            if !["chrono", "time", "jiff"].contains(&identity.package.as_str()) { return None; }
            let role = context.find_role_config(&index.crate_name).and_then(|config| config.role);
            if matches!(role, Some(Role::Technology | Role::Contract)) || context.is_in_technology_module(ordinal, reference.module) { return None; }
            Some(build_node_finding("ForbiddenImport", &reference.file, reference.anchor, identity.format_path(), "A time crate belongs only at a technology or wire-contract boundary; expose SystemTime and Duration (time-api.rs.md)".to_owned()))
        })).collect()
    }
}
