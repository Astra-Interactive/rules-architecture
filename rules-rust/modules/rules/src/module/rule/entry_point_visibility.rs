use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, find_shape_visibility};
use syn::{Item, Visibility};

pub(crate) struct EntryPointVisibility;

impl Rule for EntryPointVisibility {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().enumerate().flat_map(|(ordinal, index)| index.items.iter().filter_map(move |item| {
            let is_type = matches!(item.syntax, Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Type(_) | Item::Trait(_));
            let kind = find_kind_name(context, ordinal, item.module);
            let public = matches!(find_shape_visibility(&item.syntax), Some(Visibility::Public(_)));
            (is_type && public && context.is_entry_kind(&index.crate_name, kind)).then(|| build_shape_item_finding(index, item, "EntryPointVisibility", "entry-point kind types are pub(crate); other crates construct the DI module"))
        })).collect()
    }
}
