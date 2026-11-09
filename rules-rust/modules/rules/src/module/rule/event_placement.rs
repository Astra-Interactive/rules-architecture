use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::{
    build_shape_item_finding, external_trait, find_kind_name, owns_implementation,
};

pub(crate) struct EventPlacement;

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
                let platform_event = platform
                    && kind == "event"
                    && index.implementations.iter().any(|implementation| {
                        owns_implementation(index, identity, implementation)
                            && external_trait(context, ordinal, implementation).is_some()
                    });
                if !platform_event {
                    findings.push(build_shape_item_finding(index, item, "EventPlacement", "Domain Event types live in model; platform event implementations stay in event of a platform crate (arch-domain.rs.md)"));
                }
            }
        }
        findings
    }
}
