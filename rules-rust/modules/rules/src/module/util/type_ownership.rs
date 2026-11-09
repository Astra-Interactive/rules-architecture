use crate::check::model::RuleContext;
use crate::module::util::owns_implementation;
use crate::source::model::{WorkspaceUse, WorkspaceUseKind};
use std::collections::BTreeSet;

pub(crate) fn declaration_owner(
    context: &RuleContext,
    usage: &WorkspaceUse,
) -> Option<(usize, usize)> {
    let index = &context.indexes[usage.source_index];
    let declaration = index.items.iter().enumerate().find(|(_, item)| {
        item.file == usage.file
            && item.span.contains(&usage.span)
            && matches!(
                item.syntax,
                syn::Item::Struct(_)
                    | syn::Item::Enum(_)
                    | syn::Item::Union(_)
                    | syn::Item::Trait(_)
            )
    });
    if let Some((identity, _)) = declaration {
        return Some((usage.source_index, identity));
    }
    let implementation = index.implementations.iter().find(|implementation| {
        implementation.file == usage.file && implementation.span.contains(&usage.span)
    })?;
    let owners: BTreeSet<_> = index
        .items
        .iter()
        .enumerate()
        .filter(|(identity, _)| owns_implementation(index, *identity, implementation))
        .map(|(identity, _)| (usage.source_index, identity))
        .collect();
    (owners.len() == 1)
        .then(|| owners.into_iter().next())
        .flatten()
}

fn field_reference(context: &RuleContext, usage: &WorkspaceUse, owner: (usize, usize)) -> bool {
    if usage.kind != WorkspaceUseKind::Type || owner.0 != usage.source_index {
        return false;
    }
    let item = &context.indexes[owner.0].items[owner.1];
    let fields: Vec<_> = match &item.syntax {
        syn::Item::Struct(value) => value.fields.iter().collect(),
        syn::Item::Enum(value) => value
            .variants
            .iter()
            .flat_map(|variant| &variant.fields)
            .collect(),
        syn::Item::Union(value) => value.fields.named.iter().collect(),
        _ => return false,
    };
    fields.iter().any(|field| {
        crate::source::util::find_node_span(syn::spanned::Spanned::span(&field.ty))
            .contains(&usage.span)
    })
}

fn alias_reference(context: &RuleContext, usage: &WorkspaceUse) -> bool {
    context.indexes[usage.source_index]
        .items
        .iter()
        .any(|item| {
            item.file == usage.file
                && item.span.contains(&usage.span)
                && matches!(item.syntax, syn::Item::Type(_))
        })
}

pub(crate) fn exclusive_field_owner(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> Option<(usize, usize)> {
    if context.name_index().has_blind_spots {
        return None;
    }
    let mut owners = BTreeSet::new();
    let mut fields = BTreeSet::new();
    for usage in context
        .name_index()
        .users_of(ordinal, identity)
        .filter(|usage| !usage.is_test && usage.cfg.is_satisfiable())
    {
        if usage.ambiguous {
            return None;
        }
        if usage.kind == WorkspaceUseKind::Import || alias_reference(context, usage) {
            continue;
        }
        let owner = declaration_owner(context, usage)?;
        if owner == (ordinal, identity) {
            continue;
        }
        owners.insert(owner);
        if field_reference(context, usage, owner) {
            fields.insert(owner);
        }
    }
    (owners.len() == 1 && fields == owners)
        .then(|| owners.into_iter().next())
        .flatten()
}

pub(crate) fn has_type_behaviour(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
    let index = &context.indexes[ordinal];
    index.implementations.iter().any(|implementation| {
        owns_implementation(index, identity, implementation)
            && implementation
                .syntax
                .items
                .iter()
                .any(|item| matches!(item, syn::ImplItem::Fn(_)))
    })
}
