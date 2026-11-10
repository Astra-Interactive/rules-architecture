use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
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

    fn is_model_parameter(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        typed: &syn::Type,
    ) -> bool {
        let function_cfg = item.cfg.in_scope(ordinal);
        context
            .resolve_outer_workspace_type(ordinal, item.module, typed)
            .iter()
            .any(|target| Self::is_model_target(context, target, &function_cfg))
    }
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
