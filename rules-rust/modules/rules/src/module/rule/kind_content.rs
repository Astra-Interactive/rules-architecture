use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name};
use crate::source::model::SourceTree;
use syn::Item;

pub(crate) struct KindContent;

impl KindContent {
    fn is_outside_row(kind: &str, item: &Item) -> bool {
        let is_value_item = matches!(item, Item::Fn(_) | Item::Static(_));
        match kind {
            "api" => {
                is_value_item
                    || matches!(
                        item,
                        Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Const(_)
                    )
            }
            "model" => is_value_item,
            "usecase" => {
                is_value_item || matches!(item, Item::Enum(_) | Item::Union(_) | Item::Const(_))
            }
            _ => false,
        }
    }
}

impl Rule for KindContent {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index.items.iter().filter_map(move |item| {
                    let is_platform_code = item.cfg.mentions_target();
                    if index.modules[item.module].tree != SourceTree::Src || is_platform_code {
                        return None;
                    }
                    let kind = find_kind_name(context, ordinal, item.module);
                    Self::is_outside_row(kind, &item.syntax).then(|| {
                        build_shape_item_finding(
                            index,
                            item,
                            "KindContent",
                            "api holds contracts, model the level's types, usecase its operations; functions live in util (arch-package.md)",
                        )
                    })
                })
            })
            .collect()
    }
}
