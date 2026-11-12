use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::rule::AssociatedFactoryFunction;
use crate::module::util::{build_shape_item_finding, find_kind_name, is_model_item};
use crate::source::model::{CfgPredicate, IndexedItem, WorkspaceResolvedPath};
use syn::{FnArg, Item};

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
        is_model_item(
            context,
            target.index,
            &context.indexes[target.index].items[identity],
        ) && target.path.cfg.and(function_cfg).is_satisfiable()
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

    fn is_model_parameter(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        typed: &syn::Type,
    ) -> bool {
        let function_cfg = item.cfg.in_scope(ordinal);
        let pointee = Self::peel_pointers(context, ordinal, item.module, typed);
        [typed, pointee].iter().any(|candidate| {
            context
                .resolve_outer_workspace_type(ordinal, item.module, candidate)
                .iter()
                .any(|target| Self::is_model_target(context, target, &function_cfg))
        })
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
        function.sig.ident.to_string().starts_with("create_")
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
}

impl Rule for UtilFunctionAsExtension {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index.items.iter().filter_map(move |item| {
                    let Item::Fn(function) = &item.syntax else {
                        return None;
                    };
                    if find_kind_name(context, ordinal, item.module) != "util"
                        || function.sig.inputs.len() != 1
                        || Self::is_aggregate_factory(context, ordinal, item, function)
                    {
                        return None;
                    }
                    let Some(FnArg::Typed(parameter)) = function.sig.inputs.first() else {
                        return None;
                    };
                    Self::is_model_parameter(context, ordinal, item, &parameter.ty).then(|| {
                        build_shape_item_finding(
                            index,
                            item,
                            "UtilFunctionAsExtension",
                            "a one-model rule is an inherent extension method in util",
                        )
                    })
                })
            })
            .collect()
    }
}
