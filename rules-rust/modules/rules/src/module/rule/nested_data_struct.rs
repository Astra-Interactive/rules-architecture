use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{
    build_shape_item_finding, exclusive_field_owner, field_owners, has_type_behaviour,
    returned_to_other_owner,
};

pub(crate) struct NestedDataStruct;

impl Rule for NestedDataStruct {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().enumerate().flat_map(|(ordinal, index)| index.items.iter().enumerate().filter_map(move |(identity, item)| {
            if !matches!(item.syntax, syn::Item::Struct(_)) || has_type_behaviour(context, ordinal, identity) { return None; }
            let own_model_file = crate::module::util::find_kind_name(context, ordinal, item.module) == "model" && std::path::Path::new(&item.file).file_name().and_then(|name| name.to_str()) == Some(format!("{}.rs", crate::syntax::util::to_snake_case(&item.name)).as_str());
            if returned_to_other_owner(context, ordinal, identity) { return (!own_model_file).then(|| build_shape_item_finding(index, item, "NestedDataStruct", "a returned data type with a proven different declared consumer has its own file in model")); }
            if field_owners(context, ordinal, identity).len() > 1 { return (!own_model_file).then(|| build_shape_item_finding(index, item, "NestedDataStruct", "a plain data type used as a field by several types has its own file in model")); }
            let owner = exclusive_field_owner(context, ordinal, identity)?;
            let declaration = &context.indexes[owner.0].items[owner.1];
            (owner.0 == ordinal && declaration.file != item.file).then(|| build_shape_item_finding(index, item, "NestedDataStruct", "a plain data type used exclusively by one field owner belongs in that owner's file"))
        })).collect()
    }
}
