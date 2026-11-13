use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, item_concept};
use crate::source::model::{IndexedImpl, SourceTree};
use std::collections::BTreeSet;

pub(crate) struct ImplementationPlacement;

impl ImplementationPlacement {
    fn list_contract_concepts(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<Vec<String>> {
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return Vec::new();
        };
        context
            .resolve_workspace_path(ordinal, implementation.module, path)
            .into_iter()
            .filter_map(|target| {
                let item = &context.indexes[target.index].items[target.path.item?];
                let is_contract = matches!(item.syntax, syn::Item::Trait(_))
                    && find_kind_name(context, target.index, item.module) == "api";
                is_contract.then(|| item_concept(context, target.index, item.module))
            })
            .collect()
    }

    fn list_misplaced(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<usize> {
        let index = &context.indexes[ordinal];
        let contracts = Self::list_contract_concepts(context, ordinal, implementation);
        if contracts.is_empty() {
            return Vec::new();
        }
        index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .into_iter()
            .filter(|identity| {
                let item = &index.items[*identity];
                let kind = find_kind_name(context, ordinal, item.module);
                let concept = item_concept(context, ordinal, item.module);
                index.modules[item.module].tree == SourceTree::Src
                    && matches!(item.syntax, syn::Item::Struct(_) | syn::Item::Enum(_))
                    && context.is_technology_kind(&index.crate_name, kind)
                    && contracts.iter().all(|contract| *contract != concept)
            })
            .collect()
    }
}

impl Rule for ImplementationPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            let misplaced: BTreeSet<usize> = index
                .implementations
                .iter()
                .flat_map(|implementation| Self::list_misplaced(context, ordinal, implementation))
                .collect();
            findings.extend(misplaced.into_iter().map(|identity| {
                build_shape_item_finding(
                    index,
                    &index.items[identity],
                    "ImplementationPlacement",
                    "a contract's implementation sits under the concept of its contract (arch-package.md)",
                )
            }));
        }
        findings
    }
}
