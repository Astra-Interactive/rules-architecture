use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, resolve_shape_type};
use syn::{FnArg, Item};

pub(crate) struct UtilFunctionAsExtension;

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
                    let model = resolve_shape_type(index, item.module, &parameter.ty)
                        .iter()
                        .any(|target| {
                            let target = &index.items[*target];
                            matches!(
                                target.syntax,
                                Item::Struct(_) | Item::Enum(_) | Item::Union(_)
                            ) && find_kind_name(context, ordinal, target.module) == "model"
                                && target.cfg.and(&item.cfg).is_satisfiable()
                        });
                    model.then(|| {
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
