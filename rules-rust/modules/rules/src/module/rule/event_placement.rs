use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::{
    build_shape_item_finding, external_trait, find_kind_name, owns_implementation,
};
use crate::source::model::IndexedItem;
use crate::source::model::Resolution;
use crate::source::util::list_derive_paths;

pub(crate) struct EventPlacement;

impl EventPlacement {
    pub(crate) fn is_foreign_path(path: &str) -> bool {
        path.split_once("::")
            .is_some_and(|(package, _)| !["std", "core", "alloc"].contains(&package))
    }

    fn is_host_path(context: &RuleContext, ordinal: usize, path: &str) -> bool {
        let identity = context.resolve_external_identity(ordinal, path);
        context.is_platform_identity(&identity)
    }

    pub(crate) fn is_workspace_path(context: &RuleContext, ordinal: usize, path: &str) -> bool {
        let identity = context.resolve_external_identity(ordinal, path);
        let package = identity.package.replace('-', "_");
        context
            .workspace
            .members
            .iter()
            .any(|member| member.name.replace('-', "_") == package)
    }

    pub(crate) fn is_host_mark_path(context: &RuleContext, ordinal: usize, path: &str) -> bool {
        if context.project_config.platform_words.is_empty() {
            return Self::is_foreign_path(path) && !Self::is_workspace_path(context, ordinal, path);
        }
        Self::is_host_path(context, ordinal, path)
    }

    fn is_host_trait_path(context: &RuleContext, ordinal: usize, path: &str) -> bool {
        context.project_config.platform_words.is_empty()
            || Self::is_host_path(context, ordinal, path)
    }

    pub(crate) fn has_host_derive(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let attributes = match &item.syntax {
            syn::Item::Struct(value) => &value.attrs,
            syn::Item::Enum(value) => &value.attrs,
            syn::Item::Union(value) => &value.attrs,
            _ => return false,
        };
        list_derive_paths(attributes).iter().any(|path| {
            match index.resolve_path(item.module, path) {
                Resolution::External(path) => Self::is_host_mark_path(context, ordinal, &path),
                _ => false,
            }
        })
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
                        && external_trait(context, ordinal, implementation)
                            .is_some_and(|path| Self::is_host_trait_path(context, ordinal, &path))
                });
                let platform_event = platform
                    && kind == "event"
                    && (implements_host_trait || Self::has_host_derive(context, ordinal, item));
                if !platform_event {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "EventPlacement",
                        "a type named …Event lives in model (arch-domain.md, Domain event); only a platform's event type stays in event of a platform crate",
                    ));
                }
            }
        }
        findings
    }
}
