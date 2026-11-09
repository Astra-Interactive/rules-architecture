use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::{
    build_shape_item_finding, build_shape_module_finding, find_shape_visibility,
};
use syn::{Item, Visibility};

pub(crate) struct EntryPointShape;

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
            let cdylib = context
                .workspace
                .members
                .iter()
                .find(|member| member.name == index.crate_name)
                .is_some_and(|member| {
                    member
                        .targets
                        .iter()
                        .any(|target| target.kinds.iter().any(|kind| kind == "cdylib"))
                });
            for item in index.items.iter().filter(|item| item.module == 0) {
                let public = matches!(
                    find_shape_visibility(&item.syntax),
                    Some(Visibility::Public(_))
                );
                let public_main =
                    item.name == "main" && matches!(item.syntax, Item::Fn(_)) && public;
                let hidden_host =
                    cdylib && config.host_items.iter().any(|host| host == &item.name) && !public;
                if public_main || hidden_host {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "EntryPointShape",
                        "main is private; a cdylib's configured host items are public",
                    ));
                }
            }
        }
        findings
    }
}
