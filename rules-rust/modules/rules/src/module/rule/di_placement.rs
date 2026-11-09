use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, build_shape_module_finding, find_kind_name};

pub(crate) struct DiPlacement;

impl DiPlacement {
    fn constructs_dependency(
        context: &RuleContext,
        ordinal: usize,
        usage: &crate::source::model::WorkspaceUse,
    ) -> bool {
        let eligible = usage.source_index == ordinal
            && !usage.is_test
            && !usage.ambiguous
            && crate::module::util::is_construction_use(context, usage)
            && crate::module::util::workspace_implementation_under(
                context,
                usage.target_index,
                usage.target_item,
                &usage.cfg,
            );
        if !eligible {
            return false;
        }
        let index = &context.indexes[ordinal];
        !index.implementations.iter().any(|implementation| {
            implementation.file == usage.file
                && implementation.span.contains(&usage.span)
                && ordinal == usage.target_index
                && crate::module::util::owns_implementation(
                    index,
                    usage.target_item,
                    implementation,
                )
        })
    }

    fn missing_module(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        if index
            .items
            .iter()
            .any(|item| crate::module::util::is_di_type(context, ordinal, item))
        {
            return Vec::new();
        }
        let mut findings: Vec<_> = index
            .items
            .iter()
            .enumerate()
            .filter(|(identity, item)| {
                index.modules[item.module].tree == crate::source::model::SourceTree::Src
                    && crate::module::util::lifecycle_implementation(context, ordinal, *identity)
            })
            .map(|(_, item)| {
                build_shape_item_finding(
                    index,
                    item,
                    "DiPlacement",
                    "a resource with lifecycle has an owning DI module in its crate",
                )
            })
            .collect();
        if let Some(usage) = context
            .name_index()
            .uses
            .iter()
            .find(|usage| Self::constructs_dependency(context, ordinal, usage))
        {
            findings.push(crate::check::util::build_node_finding(
                "DiPlacement",
                &usage.file,
                usage.anchor,
                index.crate_name.clone(),
                "a crate that assembles behavioral dependencies has its own DI module".to_owned(),
            ));
        }
        findings
    }

    fn only_contracts(context: &RuleContext, ordinal: usize) -> bool {
        let index = &context.indexes[ordinal];
        let data = index
            .items
            .iter()
            .any(|item| ["api", "model"].contains(&find_kind_name(context, ordinal, item.module)));
        data && index.items.iter().all(|item| {
            ["api", "model", "di"].contains(&find_kind_name(context, ordinal, item.module))
        }) && index
            .modules
            .iter()
            .filter(|module| module.parent == Some(0))
            .all(|module| {
                ["api", "model", "di"]
                    .contains(&module.module.path.last().map(String::as_str).unwrap_or(""))
            })
    }
}

impl Rule for DiPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                if matches!(item.syntax, syn::Item::Struct(_))
                    && item.name.ends_with("Module")
                    && find_kind_name(context, ordinal, item.module) != "di"
                {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "DiPlacement",
                        "dependency wiring structs live in di",
                    ));
                }
            }
            findings.extend(Self::missing_module(context, ordinal));
            if !Self::only_contracts(context, ordinal) {
                continue;
            }
            for module in &index.modules {
                if !module.is_file_module
                    && module.module.path.last().is_some_and(|name| name == "di")
                {
                    findings.extend(build_shape_module_finding(
                        context,
                        ordinal,
                        module.id,
                        "DiPlacement",
                        "a contracts-and-models-only crate has no dependency wiring module",
                    ));
                }
            }
        }
        findings
    }
}
