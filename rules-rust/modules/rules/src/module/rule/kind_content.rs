use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, owns_implementation};
use crate::source::model::{CrateIndex, IndexedImpl, IndexedItem, SourceTree};
use syn::{ImplItem, Item};

pub(crate) struct KindContent;

impl KindContent {
    fn is_value_item(item: &Item) -> bool {
        matches!(item, Item::Fn(_) | Item::Static(_))
    }

    fn is_type_item(item: &Item) -> bool {
        matches!(item, Item::Struct(_) | Item::Enum(_) | Item::Union(_))
    }

    fn is_outside_row(kind: &str, item: &Item) -> bool {
        let is_value_item = Self::is_value_item(item);
        match kind {
            "api" => is_value_item || Self::is_type_item(item) || matches!(item, Item::Const(_)),
            "model" => is_value_item,
            "usecase" => {
                is_value_item || matches!(item, Item::Enum(_) | Item::Union(_) | Item::Const(_))
            }
            _ => false,
        }
    }

    fn is_event_type_name(name: &str) -> bool {
        name.ends_with("Listener") || name.ends_with("Event")
    }

    fn is_production_impl(index: &CrateIndex, implementation: &IndexedImpl) -> bool {
        let is_source = index.modules[implementation.module].tree == SourceTree::Src;
        is_source && !implementation.cfg.requires_test()
    }

    fn is_plain_data(index: &CrateIndex, identity: usize) -> bool {
        !index.implementations.iter().any(|implementation| {
            let holds_function = implementation
                .syntax
                .items
                .iter()
                .any(|member| matches!(member, ImplItem::Fn(_)));
            if !holds_function || !Self::is_production_impl(index, implementation) {
                return false;
            }
            owns_implementation(index, identity, implementation)
        })
    }

    fn declares_event_type(index: &CrateIndex, item: &IndexedItem) -> bool {
        index.items.iter().any(|sibling| {
            let is_beside = sibling.module == item.module && sibling.file == item.file;
            is_beside
                && Self::is_type_item(&sibling.syntax)
                && Self::is_event_type_name(&sibling.name)
        })
    }

    fn is_outside_event_row(index: &CrateIndex, identity: usize) -> bool {
        let item = &index.items[identity];
        if Self::is_value_item(&item.syntax) {
            return true;
        }
        if !Self::is_type_item(&item.syntax) || Self::is_event_type_name(&item.name) {
            return false;
        }
        if !Self::is_plain_data(index, identity) {
            return true;
        }
        !Self::declares_event_type(index, item)
    }

    fn find_row_message(index: &CrateIndex, identity: usize, kind: &str) -> Option<&'static str> {
        if kind == "event" {
            return Self::is_outside_event_row(index, identity).then_some(Self::EVENT_MESSAGE);
        }
        Self::is_outside_row(kind, &index.items[identity].syntax).then_some(Self::ROW_MESSAGE)
    }

    const ROW_MESSAGE: &str = "api holds contracts, model the level's types, usecase its operations; functions live in util (arch-package.md)";
    const EVENT_MESSAGE: &str = "event holds listeners and platform event types: a job goes to job or service, state behind a contract in api, a flow of several steps to usecase, a platform effect to util (arch-package.md, Concepts and kinds 8)";
}

impl Rule for KindContent {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(move |(identity, item)| {
                        let is_platform_code = item.cfg.mentions_target();
                        let is_outside_source = index.modules[item.module].tree != SourceTree::Src;
                        if is_outside_source || is_platform_code {
                            return None;
                        }
                        let kind = find_kind_name(context, ordinal, item.module);
                        let message = Self::find_row_message(index, identity, kind)?;
                        Some(build_shape_item_finding(
                            index,
                            item,
                            "KindContent",
                            message,
                        ))
                    })
            })
            .collect()
    }
}
