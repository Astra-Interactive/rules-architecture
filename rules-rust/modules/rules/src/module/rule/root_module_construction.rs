use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::rule::RootConstructionVisitor;
use crate::module::util::is_di_type;
use crate::module::util::owns_implementation;

pub(crate) struct RootModuleConstruction;

impl RootModuleConstruction {
    fn check_item(context: &RuleContext, ordinal: usize, identity: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let item = &index.items[identity];
        if item.name != context.root_module_name() || !is_di_type(context, ordinal, item) {
            return Vec::new();
        }
        let mut findings = Vec::new();
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
        {
            let invalid = implementation
                .syntax
                .items
                .iter()
                .filter_map(|method| match method {
                    syn::ImplItem::Fn(method) => Some(RootConstructionVisitor::collect(
                        context,
                        ordinal,
                        implementation.module,
                        &method.block,
                    )),
                    _ => None,
                })
                .flatten();
            findings.extend(invalid.map(|span| build_node_finding("RootModuleConstruction", &implementation.file, span, item.name.clone(), "the root constructor creates child DI modules; services are assembled by those children".to_owned())));
        }
        findings
    }
}

impl Rule for RootModuleConstruction {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                (0..index.items.len())
                    .flat_map(move |identity| Self::check_item(context, ordinal, identity))
            })
            .collect()
    }
}
