use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::{
    build_shape_item_finding, external_trait, find_kind_name, owns_implementation,
};
use crate::source::model::CrateIndex;
use crate::source::model::IndexedItem;
use crate::source::model::Resolution;
use syn::punctuated::Punctuated;

pub(crate) struct EventPlacement;

impl EventPlacement {
    fn is_foreign_path(resolution: &Resolution) -> bool {
        let Resolution::External(path) = resolution else {
            return false;
        };
        path.split_once("::")
            .is_some_and(|(package, _)| !["std", "core", "alloc"].contains(&package))
    }

    fn has_foreign_derive(index: &CrateIndex, item: &IndexedItem) -> bool {
        let attributes = match &item.syntax {
            syn::Item::Struct(value) => &value.attrs,
            syn::Item::Enum(value) => &value.attrs,
            syn::Item::Union(value) => &value.attrs,
            _ => return false,
        };
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("derive"))
            .filter_map(|attribute| {
                attribute
                    .parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                    .ok()
            })
            .flatten()
            .any(|path| Self::is_foreign_path(&index.resolve_path(item.module, &path)))
    }
}

impl Rule for EventPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            let platform = context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role)
                == Some(Role::Platform);
            for (identity, item) in index.items.iter().enumerate() {
                let is_type = matches!(
                    item.syntax,
                    syn::Item::Struct(_)
                        | syn::Item::Enum(_)
                        | syn::Item::Union(_)
                        | syn::Item::Trait(_)
                );
                let kind = find_kind_name(context, ordinal, item.module);
                if !is_type || !item.name.ends_with("Event") || kind == "model" {
                    continue;
                }
                let implements_host_trait = index.implementations.iter().any(|implementation| {
                    owns_implementation(index, identity, implementation)
                        && external_trait(context, ordinal, implementation).is_some()
                });
                let platform_event = platform
                    && kind == "event"
                    && (implements_host_trait || Self::has_foreign_derive(index, item));
                if !platform_event {
                    findings.push(build_shape_item_finding(index, item, "EventPlacement", "Domain Event types live in model; platform event implementations stay in event of a platform crate (arch-domain.rs.md)"));
                }
            }
        }
        findings
    }
}
