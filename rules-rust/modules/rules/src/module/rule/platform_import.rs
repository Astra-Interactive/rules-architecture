use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::module::util::find_kind_name;
use crate::source::model::{ExternalPath, Resolution, SourceReference};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;

pub(crate) struct PlatformImport;

impl PlatformImport {
    fn is_registration_reference(
        context: &RuleContext,
        ordinal: usize,
        reference: &SourceReference,
    ) -> bool {
        context.indexes[ordinal].items.iter().any(|item| {
            item.module == reference.module
                && item.file == reference.file
                && item.span.contains(&reference.span)
                && item.cfg.and(&reference.cfg).is_satisfiable()
                && context.is_platform_registration_trait(ordinal, item)
        })
    }

    fn is_listener_implementation(
        context: &RuleContext,
        ordinal: usize,
        reference: &SourceReference,
        kind: &str,
    ) -> bool {
        kind == "event"
            && context.indexes[ordinal]
                .implementations
                .iter()
                .any(|implementation| {
                    implementation.module == reference.module
                        && implementation
                            .syntax
                            .trait_
                            .as_ref()
                            .is_some_and(|(path, _)| find_node_span(path.span()) == reference.span)
                })
    }

    fn is_allowed_library(path: &ExternalPath) -> bool {
        path.package == "serde"
            || (path.package == "tokio"
                && path
                    .segments
                    .get(1)
                    .is_some_and(|segment| segment == "sync"))
    }

    fn check_reference(
        context: &RuleContext,
        ordinal: usize,
        reference: &SourceReference,
    ) -> Option<RuleFinding> {
        let Resolution::External(path) = &reference.resolution else {
            return None;
        };
        let identity = context.resolve_external_identity(ordinal, path);
        if Self::is_allowed_library(&identity) {
            return None;
        }
        let platform = context.is_platform_identity(&identity);
        let technology = context.is_technology_identity(ordinal, &identity);
        if !platform && !technology {
            return None;
        }
        let index = &context.indexes[ordinal];
        let kind = find_kind_name(context, ordinal, reference.module);
        let role = context
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role);
        let unit_scope = matches!(
            role,
            Some(Role::CoreApi | Role::StorageContract | Role::Contract)
        ) || (context.is_plugin_product() && role == Some(Role::Api));
        let domain_scope = ["model", "api", "usecase"].contains(&kind);
        let listener = identity
            .segments
            .last()
            .is_some_and(|name| name.ends_with("Listener") || name.ends_with("Callback"));
        let boundary_scope = technology
            && listener
            && (["model", "usecase"].contains(&kind)
                || context.is_entry_kind(&index.crate_name, kind));
        if !unit_scope && !domain_scope && !boundary_scope {
            return None;
        }
        if Self::is_registration_reference(context, ordinal, reference)
            || Self::is_listener_implementation(context, ordinal, reference, kind)
        {
            return None;
        }
        Some(build_node_finding("PlatformImport", &reference.file, reference.anchor, identity.format_path(), "Keep platform and technology representations behind a contract at their boundary (arch-domain.rs.md; arch-context.md)".to_owned()))
    }
}

impl Rule for PlatformImport {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .references
                    .iter()
                    .filter_map(move |reference| Self::check_reference(context, ordinal, reference))
            })
            .collect()
    }
}
