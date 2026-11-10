use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::rule::GlobalStateVisitor;
use crate::module::util::build_shape_item_finding;

pub(crate) struct GlobalState;

impl Rule for GlobalState {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let syn::Item::Static(shared) = &item.syntax else {
                    continue;
                };
                let is_invalid = GlobalStateVisitor::invalid_static(
                    context,
                    ordinal,
                    item.module,
                    shared,
                    &item.cfg,
                );
                if is_invalid {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "GlobalState",
                        "shared state is passed from DI, never stored in a global static",
                    ));
                }
            }
            findings.extend(GlobalStateVisitor::check(context, ordinal));
        }
        findings
    }
}
