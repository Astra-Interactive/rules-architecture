use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::declaration::rule::ErrorFamilyShape;
use crate::declaration::util::derives;
use crate::module::rule::PlacementTypeVisitor;
use crate::module::util::{build_shape_item_finding, find_kind_name};
use crate::source::model::CfgPredicate;
use crate::source::model::WorkspaceResolvedPath;
use std::collections::HashSet;

pub(crate) struct ErrorPlacement;

impl ErrorPlacement {
    fn is_error(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let item = &context.indexes[ordinal].items[identity];
        if matches!(item.syntax, syn::Item::Struct(_) | syn::Item::Enum(_))
            && ErrorFamilyShape::implements_error(context, ordinal, identity)
        {
            return true;
        }
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

    pub(crate) fn find_result_error(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        output: &syn::ReturnType,
    ) -> Option<syn::Type> {
        let syn::ReturnType::Type(_, typed) = output else {
            return None;
        };
        let syn::Type::Path(path) = typed.as_ref() else {
            return None;
        };
        let is_result = context
            .canonical_external_type_path(ordinal, module, &path.path)
            .is_some_and(|identity| {
                ["std", "core"].contains(&identity.package.as_str())
                    && identity
                        .segments
                        .last()
                        .is_some_and(|name| name == "Result")
            });
        let syn::PathArguments::AngleBracketed(arguments) = &path.path.segments.last()?.arguments
        else {
            return None;
        };
        let error = arguments.args.iter().nth(1)?;
        match error {
            syn::GenericArgument::Type(error) if is_result => Some(error.clone()),
            _ => None,
        }
    }

    fn collect_signature_errors(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        output: &syn::ReturnType,
        errors: &mut HashSet<usize>,
    ) {
        let Some(error) = Self::find_result_error(context, ordinal, module, output) else {
            return;
        };
        let items = &context.indexes[ordinal].items;
        errors.extend(
            context
                .resolve_workspace_type(ordinal, module, &error)
                .iter()
                .filter(|target| target.index == ordinal)
                .filter_map(|target| target.path.item)
                .filter(|identity| matches!(items[*identity].syntax, syn::Item::Enum(_))),
        );
    }

    fn collect_result_errors(context: &RuleContext, ordinal: usize) -> HashSet<usize> {
        let index = &context.indexes[ordinal];
        let mut errors = HashSet::new();
        for item in &index.items {
            if let syn::Item::Fn(function) = &item.syntax {
                let output = &function.sig.output;
                Self::collect_signature_errors(context, ordinal, item.module, output, &mut errors);
            }
        }
        for implementation in &index.implementations {
            for member in &implementation.syntax.items {
                if let syn::ImplItem::Fn(method) = member {
                    let module = implementation.module;
                    let output = &method.sig.output;
                    Self::collect_signature_errors(context, ordinal, module, output, &mut errors);
                }
            }
        }
        errors
    }

    fn check_route_target(
        context: &RuleContext,
        ordinal: usize,
        cfg: &CfgPredicate,
        target: &WorkspaceResolvedPath,
    ) -> Option<RuleFinding> {
        let identity = target.path.item?;
        let target_index = &context.indexes[target.index];
        let target_item = &target_index.items[identity];
        if !matches!(target_item.syntax, syn::Item::Enum(_))
            || !Self::is_error(context, target.index, identity)
            || !target_item
                .cfg
                .and(cfg)
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

    fn check_route_output(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        cfg: &CfgPredicate,
        output: &syn::ReturnType,
    ) -> Vec<RuleFinding> {
        let syn::ReturnType::Type(_, result) = output else {
            return Vec::new();
        };
        Self::route_type_targets(context, ordinal, module, result, 0)
            .iter()
            .filter_map(|target| Self::check_route_target(context, ordinal, cfg, target))
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
            if let syn::Item::Fn(function) = &item.syntax {
                let output = &function.sig.output;
                findings.extend(Self::check_route_output(
                    context,
                    ordinal,
                    item.module,
                    &item.cfg,
                    output,
                ));
            }
        }
        for implementation in index.implementations.iter().filter(|implementation| {
            implementation.syntax.trait_.is_none()
                && find_kind_name(context, ordinal, implementation.module) == "route"
        }) {
            for member in &implementation.syntax.items {
                if let syn::ImplItem::Fn(method) = member {
                    let module = implementation.module;
                    let output = &method.sig.output;
                    findings.extend(Self::check_route_output(
                        context,
                        ordinal,
                        module,
                        &implementation.cfg,
                        output,
                    ));
                }
            }
        }
        findings
    }
}

impl Rule for ErrorPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            let result_errors = Self::collect_result_errors(context, ordinal);
            findings.extend(
                index
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(identity, item)| {
                        (Self::is_error(context, ordinal, *identity)
                            || result_errors.contains(identity))
                            && find_kind_name(context, ordinal, item.module) != "model"
                    })
                    .map(|(_, item)| {
                        build_shape_item_finding(index, item, "ErrorPlacement", "Error families and result enums live in model, including replacements for old error modules (error-handling.rs.md)")
                    }),
            );
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
