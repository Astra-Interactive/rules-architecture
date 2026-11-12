use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, owns_implementation};
use crate::source::model::{IndexedImpl, IndexedItem};

pub(crate) struct InMemoryPlacement;

impl InMemoryPlacement {
    fn wrong_implementation(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        implementation: &IndexedImpl,
    ) -> bool {
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        context
            .resolve_workspace_path(ordinal, implementation.module, path)
            .iter()
            .any(|target| {
                let Some(contract) = target
                    .path
                    .item
                    .map(|identity| &context.indexes[target.index].items[identity])
                else {
                    return false;
                };
                matches!(contract.syntax, syn::Item::Trait(_))
                    && contract
                        .cfg
                        .and(&implementation.cfg)
                        .and(&target.path.cfg)
                        .is_satisfiable()
                    && item.name != format!("InMemory{}", contract.name)
            })
    }
}

impl Rule for InMemoryPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for (identity, item) in index.items.iter().enumerate() {
                if !matches!(item.syntax, syn::Item::Struct(_) | syn::Item::Enum(_)) {
                    continue;
                }
                let in_memory = find_kind_name(context, ordinal, item.module) == "memory";
                let mut invalid = item.name.starts_with("InMemory") && !in_memory;
                invalid |= index
                    .implementations
                    .iter()
                    .filter(|implementation| {
                        in_memory && owns_implementation(index, identity, implementation)
                    })
                    .any(|implementation| {
                        Self::wrong_implementation(context, ordinal, item, implementation)
                    });
                if invalid {
                    findings.push(build_shape_item_finding(index, item, "InMemoryPlacement", "An in-memory implementation lives in memory and is named InMemory<Trait> (arch-package.rs.md)"));
                }
            }
        }
        findings
    }
}
