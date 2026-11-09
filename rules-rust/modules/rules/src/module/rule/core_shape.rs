use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::util::{build_shape_item_finding, build_shape_module_finding};

pub(crate) struct CoreShape;

impl Rule for CoreShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            let role = context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role);
            if !matches!(
                role,
                Some(Role::Core | Role::CoreApi | Role::CorePlatform | Role::CoreUi)
            ) {
                continue;
            }
            findings.extend(
                index
                    .modules
                    .iter()
                    .filter(|module| {
                        !module.is_file_module
                            && module
                                .module
                                .path
                                .last()
                                .is_some_and(|name| name == "usecase")
                    })
                    .filter_map(|module| {
                        build_shape_module_finding(
                            context,
                            ordinal,
                            module.id,
                            "CoreShape",
                            "core contains no use-case kind",
                        )
                    }),
            );
            findings.extend(
                index
                    .items
                    .iter()
                    .filter(|item| {
                        matches!(item.syntax, syn::Item::Trait(_))
                            && item.name.ends_with("Repository")
                    })
                    .map(|item| {
                        build_shape_item_finding(
                            index,
                            item,
                            "CoreShape",
                            "core contains no repository contract",
                        )
                    }),
            );
        }
        findings
    }
}
