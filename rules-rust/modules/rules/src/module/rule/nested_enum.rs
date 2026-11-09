use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, exclusive_field_owner, has_type_behaviour};

pub(crate) struct NestedEnum;

impl Rule for NestedEnum {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().enumerate().flat_map(|(ordinal, index)| index.items.iter().enumerate().filter_map(move |(identity, item)| {
            if !matches!(item.syntax, syn::Item::Enum(_)) || has_type_behaviour(context, ordinal, identity) { return None; }
            let owner = exclusive_field_owner(context, ordinal, identity)?;
            let declaration = &context.indexes[owner.0].items[owner.1];
            (owner.0 == ordinal && declaration.file != item.file).then(|| build_shape_item_finding(index, item, "NestedEnum", "an enum used exclusively through one field owner belongs in that owner's file"))
        })).collect()
    }
}
