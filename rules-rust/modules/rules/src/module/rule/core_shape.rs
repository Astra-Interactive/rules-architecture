use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::module::util::{build_shape_item_finding, build_shape_module_finding};
use crate::source::model::{CrateIndex, SourceTree};

pub(crate) struct CoreShape;

impl CoreShape {
    fn find_use_case_modules(
        context: &RuleContext,
        ordinal: usize,
        index: &CrateIndex,
    ) -> Vec<RuleFinding> {
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
            })
            .collect()
    }

    fn find_repository_items(index: &CrateIndex) -> Vec<RuleFinding> {
        index
            .items
            .iter()
            .filter(|item| item.name.ends_with("Repository"))
            .filter_map(|item| {
                let production = index.modules[item.module].tree == SourceTree::Src;
                let message = match item.syntax {
                    syn::Item::Trait(_) => "core contains no repository contract",
                    syn::Item::Struct(_) | syn::Item::Enum(_) if production => {
                        "core contains no repository implementation"
                    }
                    _ => return None,
                };
                Some(build_shape_item_finding(index, item, "CoreShape", message))
            })
            .collect()
    }

    fn find_repository_implementations(index: &CrateIndex) -> Vec<RuleFinding> {
        index
            .implementations
            .iter()
            .filter(|implementation| index.modules[implementation.module].tree == SourceTree::Src)
            .filter_map(|implementation| {
                let (path, _) = implementation.syntax.trait_.as_ref()?;
                let name = path.segments.last()?.ident.to_string();
                name.ends_with("Repository").then(|| {
                    build_node_finding(
                        "CoreShape",
                        &implementation.file,
                        implementation.span,
                        name,
                        "core contains no repository implementation".to_owned(),
                    )
                })
            })
            .collect()
    }
}

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
            findings.extend(Self::find_use_case_modules(context, ordinal, index));
            findings.extend(Self::find_repository_items(index));
            findings.extend(Self::find_repository_implementations(index));
        }
        findings
    }
}
