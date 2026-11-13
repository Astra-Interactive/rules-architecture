use crate::check::model::RuleContext;
use crate::module::rule::PlacementTypeVisitor;
use crate::module::util::find_kind_name;
use crate::source::model::{
    CfgPredicate, CrateIndex, IndexedImpl, IndexedItem, Resolution, SourceReference,
};
use crate::source::util::find_node_span;
use std::collections::BTreeSet;
use syn::spanned::Spanned;

fn reference_targets(
    index: &CrateIndex,
    resolution: &Resolution,
    cfg: &CfgPredicate,
    depth: usize,
) -> BTreeSet<usize> {
    if depth > 8 {
        return BTreeSet::new();
    }
    let Resolution::Internal(paths) = resolution else {
        return BTreeSet::new();
    };
    let mut result = BTreeSet::new();
    for path in paths {
        let Some(identity) = path.item else {
            continue;
        };
        let item = &index.items[identity];
        let active = cfg.and(&path.cfg).and(&item.cfg);
        if !active.is_satisfiable() {
            continue;
        }
        if let syn::Item::Type(alias) = &item.syntax {
            let span = find_node_span(alias.ty.span());
            for reference in index.references.iter().filter(|reference| {
                reference.file == item.file
                    && reference.span.start >= span.start
                    && reference.span.end <= span.end
            }) {
                result.extend(reference_targets(
                    index,
                    &reference.resolution,
                    &active.and(&reference.cfg),
                    depth + 1,
                ));
            }
        } else if matches!(
            item.syntax,
            syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Union(_) | syn::Item::Trait(_)
        ) {
            result.insert(identity);
        }
    }
    result
}

pub(crate) fn reference_types(index: &CrateIndex, reference: &SourceReference) -> BTreeSet<usize> {
    reference_targets(index, &reference.resolution, &reference.cfg, 0)
}

pub(crate) fn is_model_item(context: &RuleContext, ordinal: usize, item: &IndexedItem) -> bool {
    find_kind_name(context, ordinal, item.module) == "model"
        && matches!(
            item.syntax,
            syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Union(_)
        )
}

pub(crate) fn item_concept(context: &RuleContext, ordinal: usize, module: usize) -> Vec<String> {
    let index = &context.indexes[ordinal];
    let Some(kind) = context.find_kind_module(ordinal, module) else {
        return Vec::new();
    };
    let mut path = index.modules[kind].module.path.clone();
    path.pop();
    if path
        .first()
        .is_some_and(|name| name == "test" || name == "fixture")
    {
        path.remove(0);
    }
    path
}

pub(crate) fn owns_implementation(
    index: &CrateIndex,
    identity: usize,
    implementation: &IndexedImpl,
) -> bool {
    index.items[identity]
        .cfg
        .and(&implementation.cfg)
        .is_satisfiable()
        && index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .contains(&identity)
}

pub(crate) fn is_try_from_impl(index: &CrateIndex, implementation: &IndexedImpl) -> bool {
    let Some((path, _)) = &implementation.syntax.trait_ else {
        return false;
    };
    match index.resolve_path(implementation.module, path) {
        Resolution::Local => {
            path.segments.len() == 1
                && path
                    .segments
                    .last()
                    .is_some_and(|last| last.ident == "TryFrom")
        }
        Resolution::External(path) => {
            ["std::convert::TryFrom", "core::convert::TryFrom"].contains(&path.as_str())
        }
        _ => false,
    }
}

pub(crate) fn list_try_from_impls(index: &CrateIndex, identity: usize) -> Vec<&IndexedImpl> {
    let item = &index.items[identity];
    index
        .implementations
        .iter()
        .filter(|implementation| {
            implementation.file == item.file
                && is_try_from_impl(index, implementation)
                && owns_implementation(index, identity, implementation)
        })
        .collect()
}

pub(crate) fn external_trait(
    context: &RuleContext,
    ordinal: usize,
    implementation: &IndexedImpl,
) -> Option<String> {
    let (path, _) = implementation.syntax.trait_.as_ref()?;
    if !context
        .resolve_workspace_path(ordinal, implementation.module, path)
        .is_empty()
    {
        return None;
    }
    match context.indexes[ordinal].resolve_path(implementation.module, path) {
        Resolution::External(path) => Some(path),
        _ => None,
    }
}

pub(crate) fn resolve_type_uses(
    index: &CrateIndex,
    module: usize,
    value: &syn::Type,
    depth: usize,
) -> BTreeSet<usize> {
    if depth > 8 {
        return BTreeSet::new();
    }
    let mut types = BTreeSet::new();
    for path in PlacementTypeVisitor::collect_paths(value) {
        let Resolution::Internal(targets) = index.resolve_path(module, &path) else {
            continue;
        };
        for target in targets {
            let Some(identity) = target.item else {
                continue;
            };
            let item = &index.items[identity];
            if let syn::Item::Type(alias) = &item.syntax {
                types.extend(resolve_type_uses(index, item.module, &alias.ty, depth + 1));
            } else {
                types.insert(identity);
            }
        }
    }
    types
}
