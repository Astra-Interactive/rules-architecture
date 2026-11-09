use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::declaration::util::has_kind;
use crate::source::model::CrateIndex;
use crate::source::model::IndexedImpl;
use crate::source::model::IndexedItem;
use crate::source::model::Resolution;
use crate::source::model::WorkspaceResolvedPath;
use crate::source::util::find_node_span;

pub(crate) struct ContractShape;

impl ContractShape {
    fn is_async_trait(value: &syn::ItemTrait) -> bool {
        value.items.iter().any(
            |item| matches!(item, syn::TraitItem::Fn(function) if function.sig.asyncness.is_some()),
        )
    }

    fn has_async_trait(index: &CrateIndex, module: usize, attributes: &[syn::Attribute]) -> bool {
        attributes.iter().any(|attribute| {
            let written = attribute.path().segments.last().is_some_and(|segment| segment.ident == "async_trait");
            let resolved = matches!(index.resolve_path(module, attribute.path()), Resolution::External(path) if path == "async_trait::async_trait");
            written || resolved
        })
    }

    fn find_bound(
        index: &CrateIndex,
        module: usize,
        bound: &syn::TypeParamBound,
    ) -> Option<String> {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return None;
        };
        if bound.modifiers.require_empty().is_err() || bound.lifetimes.is_some() {
            return None;
        }
        match index.resolve_path(module, &bound.path) {
            Resolution::External(path)
                if path == "std::marker::Send" || path == "core::marker::Send" =>
            {
                Some("Send".to_owned())
            }
            Resolution::External(path)
                if path == "std::marker::Sync" || path == "core::marker::Sync" =>
            {
                Some("Sync".to_owned())
            }
            Resolution::Local if bound.path.is_ident("Send") => Some("Send".to_owned()),
            Resolution::Local if bound.path.is_ident("Sync") => Some("Sync".to_owned()),
            _ => None,
        }
    }

    fn check_trait(index: &CrateIndex, item: &IndexedItem) -> Option<RuleFinding> {
        let syn::Item::Trait(value) = &item.syntax else {
            return None;
        };
        let bounds = value
            .supertraits
            .iter()
            .map(|bound| Self::find_bound(index, item.module, bound))
            .collect::<Vec<_>>();
        let has_send = bounds.iter().any(|bound| bound.as_deref() == Some("Send"));
        let has_sync = bounds.iter().any(|bound| bound.as_deref() == Some("Sync"));
        let invalid_api = has_kind(index, item.module, "api")
            && (!value.generics.params.is_empty()
                || value.generics.where_clause.is_some()
                || !has_send
                || !has_sync
                || bounds.iter().any(Option::is_none));
        let missing_async =
            Self::is_async_trait(value) && !Self::has_async_trait(index, item.module, &value.attrs);
        if !invalid_api && !missing_async {
            return None;
        }
        Some(build_node_finding("ContractShape", &item.file, find_node_span(value.ident.span()), format!("{}::{}", format_module_path(&index.crate_name, &index.modules[item.module].module.path), item.name), "Declare contracts as non-generic Send + Sync traits, with async_trait for async methods (arch-domain.rs.md)".to_owned()))
    }

    fn is_async_target(
        context: &RuleContext,
        implementation: &IndexedImpl,
        target: &WorkspaceResolvedPath,
    ) -> bool {
        let index = &context.indexes[target.index];
        let Some(item) = target
            .path
            .item
            .and_then(|identity| index.items.get(identity))
        else {
            return false;
        };
        item.cfg
            .and(&implementation.cfg)
            .and(&target.path.cfg)
            .is_satisfiable()
            && matches!(&item.syntax, syn::Item::Trait(value) if Self::is_async_trait(value))
    }
}

impl Rule for ContractShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (index_number, index) in context.indexes.iter().enumerate() {
            findings.extend(
                index
                    .items
                    .iter()
                    .filter_map(|item| Self::check_trait(index, item)),
            );
            for item in &index.items {
                let syn::Item::Trait(contract) = &item.syntax else {
                    continue;
                };
                if has_kind(index, item.module, "api") && contract.items.iter().any(|method| matches!(method, syn::TraitItem::Fn(method) if crate::declaration::util::invalid_contract_method(context, index_number, item, method))) {
                    findings.push(crate::module::util::build_shape_item_finding(index, item, "ContractShape", "API contracts expose static BoxStream events, domain values and async results; callbacks, raw streams and task handles remain at their boundary"));
                }
            }
            for implementation in &index.implementations {
                let Some((path, _)) = &implementation.syntax.trait_ else {
                    continue;
                };
                if Self::has_async_trait(index, implementation.module, &implementation.syntax.attrs)
                {
                    continue;
                }
                let targets =
                    context.resolve_workspace_path(index_number, implementation.module, path);
                let implements_async = targets
                    .iter()
                    .any(|target| Self::is_async_target(context, implementation, target));
                if implements_async {
                    findings.push(build_node_finding("ContractShape", &implementation.file, implementation.span, format_module_path(&index.crate_name, &index.modules[implementation.module].module.path), "Apply async_trait to every implementation of an async contract (arch-domain.rs.md)".to_owned()));
                }
            }
        }
        findings
    }
}
