use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, build_shape_module_finding, find_kind_name};

pub(crate) struct DiPlacement;

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
            let only_contracts = index
                .modules
                .iter()
                .filter(|module| !module.is_file_module)
                .filter(|module| context.find_kind_module(ordinal, module.id) == Some(module.id))
                .all(|module| {
                    ["api", "model", "di"]
                        .contains(&module.module.path.last().map(String::as_str).unwrap_or(""))
                });
            if !only_contracts {
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
