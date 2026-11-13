use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::rule::AssociatedFactoryFunction;
use crate::module::util::{
    build_shape_item_finding, declares_type_elsewhere, find_kind_name, format_shape_subject,
    is_model_item, owns_implementation,
};
use crate::source::model::{
    CfgPredicate, CrateIndex, IndexedImpl, IndexedItem, Resolution, SourceTree,
    WorkspaceResolvedPath,
};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, Item};

pub(crate) struct UtilFunctionAsExtension;

impl UtilFunctionAsExtension {
    fn is_model_target(
        context: &RuleContext,
        target: &WorkspaceResolvedPath,
        function_cfg: &CfgPredicate,
    ) -> bool {
        let Some(identity) = target.path.item else {
            return false;
        };
        let item = &context.indexes[target.index].items[identity];
        is_model_item(context, target.index, item)
            && !item.name.ends_with("Id")
            && target.path.cfg.and(function_cfg).is_satisfiable()
    }

    fn find_pointee<'typed>(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &'typed syn::Type,
    ) -> Option<&'typed syn::Type> {
        let syn::Type::Path(path) = typed else {
            return None;
        };
        let identity = context.canonical_external_type_path(ordinal, module, &path.path)?;
        let segments = identity
            .segments
            .iter()
            .skip(1)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let is_pointer = ["std", "alloc"].contains(&identity.package.as_str())
            && Self::SMART_POINTERS.contains(&segments.as_slice());
        if !is_pointer {
            return None;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &path.path.segments.last()?.arguments
        else {
            return None;
        };
        arguments.args.iter().find_map(|argument| match argument {
            syn::GenericArgument::Type(inner) => Some(inner),
            _ => None,
        })
    }

    fn peel_pointers<'typed>(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &'typed syn::Type,
    ) -> &'typed syn::Type {
        let mut current = typed;
        loop {
            let next = match current {
                syn::Type::Reference(reference) => Some(reference.elem.as_ref()),
                syn::Type::Paren(paren) => Some(paren.elem.as_ref()),
                other => Self::find_pointee(context, ordinal, module, other),
            };
            let Some(next) = next else {
                return current;
            };
            current = next;
        }
    }

    fn find_model_targets(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        function_cfg: &CfgPredicate,
        typed: &syn::Type,
    ) -> Vec<WorkspaceResolvedPath> {
        let pointee = Self::peel_pointers(context, ordinal, module, typed);
        [typed, pointee]
            .iter()
            .flat_map(|candidate| context.resolve_outer_workspace_type(ordinal, module, candidate))
            .filter(|target| Self::is_model_target(context, target, function_cfg))
            .collect()
    }

    fn list_parameter_models<'ast>(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        function_cfg: &CfgPredicate,
        inputs: impl Iterator<Item = &'ast FnArg>,
    ) -> Vec<Vec<WorkspaceResolvedPath>> {
        inputs
            .filter_map(|input| match input {
                FnArg::Typed(parameter) => Some(parameter.ty.as_ref()),
                FnArg::Receiver(_) => None,
            })
            .map(|typed| Self::find_model_targets(context, ordinal, module, function_cfg, typed))
            .filter(|targets| !targets.is_empty())
            .collect()
    }

    fn is_option(index: &CrateIndex, module: usize, path: &syn::Path) -> bool {
        match index.resolve_path(module, path) {
            Resolution::Local => path.segments.len() == 1 && path.segments[0].ident == "Option",
            Resolution::External(name) => {
                name == "std::option::Option" || name == "core::option::Option"
            }
            _ => false,
        }
    }

    fn unwrap_returned<'ast>(
        index: &CrateIndex,
        module: usize,
        typed: &'ast syn::Type,
    ) -> &'ast syn::Type {
        let syn::Type::Path(path) = typed else {
            return typed;
        };
        let wrapper = AssociatedFactoryFunction::returns_result(index, module, typed, 0)
            || Self::is_option(index, module, &path.path);
        let Some(syn::PathArguments::AngleBracketed(arguments)) =
            path.path.segments.last().map(|segment| &segment.arguments)
        else {
            return typed;
        };
        arguments
            .args
            .iter()
            .find_map(|argument| match argument {
                syn::GenericArgument::Type(inner) if wrapper => Some(inner),
                _ => None,
            })
            .unwrap_or(typed)
    }

    fn find_returned_models(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        function_cfg: &CfgPredicate,
        output: &syn::ReturnType,
    ) -> Vec<WorkspaceResolvedPath> {
        let syn::ReturnType::Type(_, typed) = output else {
            return Vec::new();
        };
        let inner = Self::unwrap_returned(&context.indexes[ordinal], module, typed);
        context
            .resolve_outer_workspace_type(ordinal, module, inner)
            .into_iter()
            .filter(|target| Self::is_model_target(context, target, function_cfg))
            .collect()
    }

    fn is_same_type(
        context: &RuleContext,
        first: &WorkspaceResolvedPath,
        second: &WorkspaceResolvedPath,
    ) -> bool {
        if first.index != second.index {
            return false;
        }
        let (Some(first_item), Some(second_item)) = (first.path.item, second.path.item) else {
            return first.path.item == second.path.item;
        };
        let items = &context.indexes[first.index].items;
        first_item == second_item
            || (items[first_item].name == items[second_item].name
                && items[first_item].module == items[second_item].module)
    }

    fn count_distinct_types(context: &RuleContext, models: &[Vec<WorkspaceResolvedPath>]) -> usize {
        let mut distinct: Vec<&WorkspaceResolvedPath> = Vec::new();
        for model in models.iter().flatten() {
            if !distinct
                .iter()
                .any(|seen| Self::is_same_type(context, seen, model))
            {
                distinct.push(model);
            }
        }
        distinct.len()
    }

    fn check_function(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        function: &syn::ItemFn,
    ) -> Option<RuleFinding> {
        if Self::is_aggregate_factory(context, ordinal, item, function) {
            return None;
        }
        let cfg = item.cfg.in_scope(ordinal);
        let models = Self::list_parameter_models(
            context,
            ordinal,
            item.module,
            &cfg,
            function.sig.inputs.iter(),
        );
        let returned =
            Self::find_returned_models(context, ordinal, item.module, &cfg, &function.sig.output);
        let transition = models.iter().flatten().any(|model| {
            returned
                .iter()
                .any(|target| Self::is_same_type(context, model, target))
        });
        let message = match (Self::count_distinct_types(context, &models), transition) {
            (0, _) | (2.., false) => return None,
            (1, false) => Self::ONE_MODEL_MESSAGE,
            (_, true) => Self::TRANSITION_MESSAGE,
        };
        Some(build_shape_item_finding(
            &context.indexes[ordinal],
            item,
            "UtilFunctionAsExtension",
            message,
        ))
    }

    fn returns_owner(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        owners: &[usize],
        output: &syn::ReturnType,
    ) -> bool {
        let syn::ReturnType::Type(_, typed) = output else {
            return false;
        };
        let index = &context.indexes[ordinal];
        let inner = Self::unwrap_returned(index, module, typed);
        matches!(inner, syn::Type::Path(path) if path.path.is_ident("Self"))
            || index
                .resolve_type_items(module, inner)
                .iter()
                .any(|identity| owners.contains(identity))
    }

    fn reads_another_model(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
        owners: &[usize],
        method: &syn::ImplItemFn,
    ) -> bool {
        let cfg = implementation
            .cfg
            .and(&CfgPredicate::from_attributes(&method.attrs))
            .in_scope(ordinal);
        let others = Self::list_parameter_models(
            context,
            ordinal,
            implementation.module,
            &cfg,
            method.sig.inputs.iter(),
        );
        others.iter().flatten().any(|model| {
            model.index != ordinal || model.path.item.is_none_or(|item| !owners.contains(&item))
        }) && !Self::returns_owner(
            context,
            ordinal,
            implementation.module,
            owners,
            &method.sig.output,
        )
    }

    fn check_implementation(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        if implementation.syntax.trait_.is_some()
            || index.modules[implementation.module].tree != SourceTree::Src
            || find_kind_name(context, ordinal, implementation.module) != "util"
            || !declares_type_elsewhere(index, implementation)
        {
            return Vec::new();
        }
        let owners: Vec<usize> = index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .into_iter()
            .filter(|identity| {
                let item = &index.items[*identity];
                is_model_item(context, ordinal, item)
                    && !item.name.ends_with("Id")
                    && owns_implementation(index, *identity, implementation)
            })
            .collect();
        if owners.is_empty() {
            return Vec::new();
        }
        implementation
            .syntax
            .items
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(method) => Some(method),
                _ => None,
            })
            .filter(|method| {
                Self::reads_another_model(context, ordinal, implementation, &owners, method)
            })
            .map(|method| {
                build_node_finding(
                    "UtilFunctionAsExtension",
                    &implementation.file,
                    find_node_span(method.span()),
                    format_shape_subject(
                        index,
                        implementation.module,
                        &method.sig.ident.to_string(),
                    ),
                    Self::TWO_MODELS_MESSAGE.to_owned(),
                )
            })
            .collect()
    }

    fn is_aggregate_factory(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        function: &syn::ItemFn,
    ) -> bool {
        let syn::ReturnType::Type(_, returned) = &function.sig.output else {
            return false;
        };
        let name = function.sig.ident.to_string();
        (name.starts_with("create_") || name.starts_with("parse_"))
            && AssociatedFactoryFunction::returns_result(
                &context.indexes[ordinal],
                item.module,
                returned,
                0,
            )
            && AssociatedFactoryFunction::returns_model(context, ordinal, item.module, returned, 0)
    }

    const SMART_POINTERS: [&'static [&'static str]; 3] =
        [&["boxed", "Box"], &["rc", "Rc"], &["sync", "Arc"]];
    const ONE_MODEL_MESSAGE: &'static str =
        "a one-model rule is an inherent extension method in util";
    const TRANSITION_MESSAGE: &'static str =
        "a transition is an inherent method of the type it returns, in util";
    const TWO_MODELS_MESSAGE: &'static str =
        "a rule that reads two model types is a free function named after the decision in util";
}

impl Rule for UtilFunctionAsExtension {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            findings.extend(index.items.iter().filter_map(|item| {
                let Item::Fn(function) = &item.syntax else {
                    return None;
                };
                let is_src = index.modules[item.module].tree == SourceTree::Src;
                if !is_src || find_kind_name(context, ordinal, item.module) != "util" {
                    return None;
                }
                Self::check_function(context, ordinal, item, function)
            }));
            findings.extend(index.implementations.iter().flat_map(|implementation| {
                Self::check_implementation(context, ordinal, implementation)
            }));
        }
        findings
    }
}
