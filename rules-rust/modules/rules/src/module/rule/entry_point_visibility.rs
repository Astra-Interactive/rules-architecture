use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::rule::{ContractPlacement, EventPlacement};
use crate::module::util::{
    build_shape_item_finding, external_trait, find_kind_name, find_shape_visibility,
    owns_implementation,
};
use syn::{Item, Visibility};

pub(crate) struct EntryPointVisibility;

impl EntryPointVisibility {
    fn has_host_trait(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        index.implementations.iter().any(|implementation| {
            owns_implementation(index, identity, implementation)
                && external_trait(context, ordinal, implementation)
                    .is_some_and(|path| EventPlacement::is_host_mark_path(context, ordinal, &path))
        })
    }

    fn is_used_by_binary(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let crate_name = &context.indexes[ordinal].crate_name;
        context
            .name_index()
            .users_of(ordinal, identity)
            .any(|usage| {
                usage.source_index != ordinal
                    && !usage.is_test
                    && context.indexes[usage.source_index].crate_name == *crate_name
            })
    }

    fn is_exempt(context: &RuleContext, ordinal: usize, identity: usize, kind: &str) -> bool {
        let item = &context.indexes[ordinal].items[identity];
        ContractPlacement::is_app_api_state_holder(context, ordinal, item)
            || Self::is_platform_event(context, ordinal, identity, kind)
            || Self::is_used_by_binary(context, ordinal, identity)
    }

    fn is_platform_event(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        kind: &str,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let is_platform = context
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role)
            == Some(Role::Platform);
        let item = &index.items[identity];
        is_platform
            && kind == "event"
            && (EventPlacement::has_host_derive(context, ordinal, item)
                || Self::has_host_trait(context, ordinal, identity))
    }
}

impl Rule for EntryPointVisibility {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().enumerate().flat_map(|(ordinal, index)| index.items.iter().enumerate().filter_map(move |(identity, item)| {
            let is_type = matches!(item.syntax, Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Type(_) | Item::Trait(_));
            let kind = find_kind_name(context, ordinal, item.module);
            let public = matches!(find_shape_visibility(&item.syntax), Some(Visibility::Public(_)));
            let reported = is_type && public && context.is_entry_kind(&index.crate_name, kind) && !Self::is_exempt(context, ordinal, identity, kind);
            reported.then(|| build_shape_item_finding(index, item, "EntryPointVisibility", "entry-point kind types are pub(crate); other crates construct the DI module"))
        })).collect()
    }
}
