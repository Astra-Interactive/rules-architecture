use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::rule::MainShapeVisitor;
use crate::module::util::build_shape_item_finding;

pub(crate) struct MainShape;

impl Rule for MainShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            if context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role)
                != Some(Role::EntryPoint)
            {
                continue;
            }
            for item in index
                .items
                .iter()
                .filter(|item| item.module == 0 && item.name == "main")
            {
                let syn::Item::Fn(function) = &item.syntax else {
                    continue;
                };
                if !MainShapeVisitor::valid(context, ordinal, item.module, &function.block) {
                    findings.push(build_shape_item_finding(index, item, "MainShape", "entry main builds a runtime, constructs/enables the root in one block_on, then disables it synchronously"));
                }
            }
        }
        findings
    }
}
