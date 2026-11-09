use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::source::model::CfgPredicate;
use crate::source::model::CrateIndex;
use crate::source::model::IndexedImpl;
use crate::source::model::IndexedItem;

pub(crate) struct NamespaceType;

impl NamespaceType {
    fn matches_type(
        index: &CrateIndex,
        implementation: &IndexedImpl,
        identity: usize,
        item: &IndexedItem,
    ) -> bool {
        index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .contains(&identity)
            && item.cfg.and(&implementation.cfg).is_satisfiable()
    }

    fn is_namespace(index: &CrateIndex, identity: usize, item: &IndexedItem) -> bool {
        let implementations = index
            .implementations
            .iter()
            .filter(|implementation| Self::matches_type(index, implementation, identity, item))
            .collect::<Vec<_>>();
        let trait_conditions = implementations
            .iter()
            .filter(|implementation| implementation.syntax.trait_.is_some())
            .map(|implementation| implementation.cfg.clone())
            .collect();
        let no_trait = CfgPredicate::Not(Box::new(CfgPredicate::Any(trait_conditions)));
        implementations.iter().any(|implementation| {
            implementation.syntax.trait_.is_none()
                && item
                    .cfg
                    .and(&implementation.cfg)
                    .and(&no_trait)
                    .is_satisfiable()
                && implementation
                    .syntax
                    .items
                    .iter()
                    .any(|item| matches!(item, syn::ImplItem::Fn(_)))
        })
    }
}

impl Rule for NamespaceType {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for index in &context.indexes {
            for (identity, item) in index.items.iter().enumerate() {
                let is_empty = match &item.syntax {
                    syn::Item::Struct(value) => value.fields.is_empty(),
                    syn::Item::Enum(value) => value.variants.is_empty(),
                    _ => false,
                };
                let generated = context
                    .crates
                    .iter()
                    .flat_map(|sources| &sources.files)
                    .any(|file| file.path == item.file && file.is_generated);
                if !is_empty || generated {
                    continue;
                }
                if Self::is_namespace(index, identity, item) {
                    findings.push(build_node_finding("NamespaceType", &item.file, item.span, format!("{}::{}", format_module_path(&index.crate_name, &index.modules[item.module].module.path), item.name), "Use module functions instead of a fieldless namespace type (arch-package.rs.md)".to_owned()));
                }
            }
        }
        findings
    }
}
