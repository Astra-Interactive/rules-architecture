use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::declaration::util::derives;
use crate::module::rule::PlacementTypeVisitor;
use crate::module::util::{build_shape_item_finding, find_kind_name};
use crate::source::model::IndexedItem;
use crate::source::model::WorkspaceResolvedPath;

pub(crate) struct ErrorPlacement;

impl ErrorPlacement {
    fn is_error(item: &IndexedItem) -> bool {
        match &item.syntax {
            syn::Item::Struct(value) => {
                item.name.ends_with("Error") || derives(&value.attrs, "Error")
            }
            syn::Item::Enum(value) => {
                item.name.ends_with("Error")
                    || item.name.ends_with("Result")
                    || derives(&value.attrs, "Error")
            }
            _ => false,
        }
    }

    fn check_route_target(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        target: &WorkspaceResolvedPath,
    ) -> Option<RuleFinding> {
        let identity = target.path.item?;
        let target_index = &context.indexes[target.index];
        let target_item = &target_index.items[identity];
        if !matches!(target_item.syntax, syn::Item::Enum(_))
            || !Self::is_error(target_item)
            || !target_item
                .cfg
                .and(&item.cfg)
                .and(&target.path.cfg)
                .is_satisfiable()
        {
            return None;
        }
        let outside = target.index != ordinal
            || find_kind_name(context, target.index, target_item.module) != "model";
        outside.then(|| build_shape_item_finding(target_index, target_item, "ErrorPlacement", "A framework route returns its entry-point crate's model error family (error-handling.rs.md)"))
    }

    fn route_type_targets(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        value: &syn::Type,
        depth: usize,
    ) -> Vec<WorkspaceResolvedPath> {
        if depth > 8 {
            return Vec::new();
        }
        PlacementTypeVisitor::collect_paths(value)
            .into_iter()
            .flat_map(|path| context.resolve_workspace_path(ordinal, module, &path))
            .flat_map(|target| Self::follow_route_target(context, target, depth))
            .collect()
    }

    fn follow_route_target(
        context: &RuleContext,
        target: WorkspaceResolvedPath,
        depth: usize,
    ) -> Vec<WorkspaceResolvedPath> {
        let Some(identity) = target.path.item else {
            return Vec::new();
        };
        let item = &context.indexes[target.index].items[identity];
        let syn::Item::Type(alias) = &item.syntax else {
            return vec![target];
        };
        Self::route_type_targets(context, target.index, item.module, &alias.ty, depth + 1)
            .into_iter()
            .map(|mut resolved| {
                resolved.path.cfg = resolved.path.cfg.and(&target.path.cfg);
                resolved
            })
            .collect()
    }

    fn route_errors(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        if context
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role)
            != Some(Role::EntryPoint)
        {
            return Vec::new();
        }
        let mut findings = Vec::new();
        for item in index
            .items
            .iter()
            .filter(|item| find_kind_name(context, ordinal, item.module) == "route")
        {
            let syn::Item::Fn(function) = &item.syntax else {
                continue;
            };
            let syn::ReturnType::Type(_, result) = &function.sig.output else {
                continue;
            };
            let targets = Self::route_type_targets(context, ordinal, item.module, result, 0);
            findings.extend(
                targets
                    .iter()
                    .filter_map(|target| Self::check_route_target(context, ordinal, item, target)),
            );
        }
        findings
    }
}

impl Rule for ErrorPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            findings.extend(index.items.iter().filter(|item| Self::is_error(item) && find_kind_name(context, ordinal, item.module) != "model").map(|item| build_shape_item_finding(index, item, "ErrorPlacement", "Error families and result enums live in model, including replacements for old error modules (error-handling.rs.md)")));
            findings.extend(Self::route_errors(context, ordinal));
        }
        findings.sort_by(|left, right| {
            left.finding
                .location
                .cmp(&right.finding.location)
                .then(left.finding.message.cmp(&right.finding.message))
        });
        findings.dedup_by(|left, right| left.finding.location == right.finding.location);
        findings
    }
}
