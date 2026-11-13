use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::build_shape_item_finding;
use crate::workspace::model::TomlValue;

pub(crate) struct RootModulePlacement;

impl Rule for RootModulePlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let name = context
            .workspace
            .root_manifest
            .root
            .find(&["workspace", "metadata", "klibs-rules", "root-module"])
            .and_then(TomlValue::as_str)
            .unwrap_or("RootModule");
        context.indexes.iter().flat_map(|index| {
            let roots: Vec<_> = index.items.iter().filter(|item| item.name == name && matches!(item.syntax, syn::Item::Struct(_))).collect();
            let member = context.workspace.members.iter().find(|member| member.name == index.crate_name);
            let entry = context.project_config.crates.iter().any(|config| config.name == index.crate_name && config.role == Some(Role::EntryPoint));
            roots.iter().filter_map(|item| {
                let correct_file = member.is_some_and(|member| item.file == format!("{}/src/di/root_module.rs", member.directory));
                let duplicate = roots.iter().any(|other| !std::ptr::eq(*item, *other) && item.cfg.and(&other.cfg).is_satisfiable());
                (!entry || !correct_file || duplicate).then(|| build_shape_item_finding(index, item, "RootModulePlacement", "one composition root lives at src/di/root_module.rs of an entry-point crate"))
            }).collect::<Vec<_>>()
        }).collect()
    }
}
