use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::{
    build_shape_item_finding, build_shape_module_finding, find_shape_visibility,
};
use crate::source::model::CrateIndex;
use crate::source::util::format_tokens;
use syn::{Item, Visibility};

pub(crate) struct EntryPointShape;

impl EntryPointShape {
    fn is_host_function(item: &Item) -> bool {
        let Item::Fn(function) = item else {
            return false;
        };
        function.attrs.iter().any(|attribute| {
            let tokens = format_tokens(&attribute.meta).replace(' ', "");
            tokens == "no_mangle"
                || tokens == "unsafe(no_mangle)"
                || tokens.starts_with("export_name=")
                || tokens.starts_with("unsafe(export_name=")
        })
    }

    fn check_items(
        index: &CrateIndex,
        host_items: &[String],
        host_library: bool,
    ) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for item in index.items.iter().filter(|item| item.module == 0) {
            let visibility = find_shape_visibility(&item.syntax);
            let is_public = matches!(visibility, Some(Visibility::Public(_)));
            let is_visible_main = item.name == "main"
                && matches!(item.syntax, Item::Fn(_))
                && matches!(
                    visibility,
                    Some(Visibility::Public(_) | Visibility::Restricted(_))
                );
            let is_hidden_host = host_library
                && !is_public
                && (host_items.contains(&item.name) || Self::is_host_function(&item.syntax));
            if is_visible_main || is_hidden_host {
                findings.push(build_shape_item_finding(
                    index,
                    item,
                    "EntryPointShape",
                    "main is private; a cdylib's configured host items are public",
                ));
            }
        }
        findings
    }
}

impl Rule for EntryPointShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            let Some(config) = context.project_config.crates.iter().find(|config| {
                config.name == index.crate_name && config.role == Some(Role::EntryPoint)
            }) else {
                continue;
            };
            for module in index
                .modules
                .iter()
                .filter(|module| module.parent == Some(0))
            {
                let name = module.module.path.last().map(String::as_str).unwrap_or("");
                if !["di", "fixture", "test"].contains(&name) {
                    findings.extend(build_shape_module_finding(
                        context,
                        ordinal,
                        module.id,
                        "EntryPointShape",
                        "an entry-point root holds only di and its fixture/test blocks",
                    ));
                }
            }
            let host_library = context
                .workspace
                .members
                .iter()
                .find(|member| member.name == index.crate_name)
                .is_some_and(|member| {
                    member.targets.iter().any(|target| {
                        target
                            .kinds
                            .iter()
                            .any(|kind| kind == "cdylib" || kind == "staticlib")
                    })
                });
            findings.extend(Self::check_items(index, &config.host_items, host_library));
        }
        findings
    }
}
