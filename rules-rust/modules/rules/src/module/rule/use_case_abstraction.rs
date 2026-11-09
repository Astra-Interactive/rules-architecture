use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::{build_shape_item_finding, find_kind_name};

pub(crate) struct UseCaseAbstraction;

impl Rule for UseCaseAbstraction {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index.items.iter().filter_map(move |item| {
                    if !matches!(item.syntax, syn::Item::Trait(_))
                        || !item.name.ends_with("UseCase")
                    {
                        return None;
                    }
                    let correct = !context.is_plugin_product()
                        && context
                            .find_role_config(&index.crate_name)
                            .and_then(|config| config.role)
                            == Some(Role::Api)
                        && find_kind_name(context, ordinal, item.module) == "usecase";
                    (!correct).then(|| {
                        build_shape_item_finding(
                            index,
                            item,
                            "UseCaseAbstraction",
                            "a use-case abstraction lives in usecase of its owner's app api unit",
                        )
                    })
                })
            })
            .collect()
    }
}
