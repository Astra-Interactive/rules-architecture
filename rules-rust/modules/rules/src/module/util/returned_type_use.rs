use crate::check::model::RuleContext;
use crate::module::util::declaration_owner;
use crate::source::model::{WorkspaceUse, WorkspaceUseKind};
use syn::spanned::Spanned;

struct ReturnedCallable {
    identity: usize,
    member: Option<String>,
    producer: Option<(usize, usize)>,
}

fn output_contains(output: &syn::ReturnType, usage: &WorkspaceUse) -> bool {
    matches!(output, syn::ReturnType::Type(_, typed) if crate::source::util::find_node_span(typed.span()).contains(&usage.span))
}

fn callable(context: &RuleContext, usage: &WorkspaceUse) -> Option<ReturnedCallable> {
    let index = &context.indexes[usage.source_index];
    for (identity, item) in index
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.file == usage.file && item.span.contains(&usage.span))
    {
        if matches!(&item.syntax, syn::Item::Fn(function) if output_contains(&function.sig.output, usage))
        {
            return Some(ReturnedCallable {
                identity,
                member: None,
                producer: None,
            });
        }
        let syn::Item::Trait(contract) = &item.syntax else {
            continue;
        };
        for method in &contract.items {
            if let syn::TraitItem::Fn(method) = method
                && output_contains(&method.sig.output, usage)
            {
                return Some(ReturnedCallable {
                    identity,
                    member: Some(method.sig.ident.to_string()),
                    producer: Some((usage.source_index, identity)),
                });
            }
        }
    }
    let owner = declaration_owner(context, usage)?;
    for implementation in index.implementations.iter().filter(|implementation| {
        implementation.file == usage.file && implementation.span.contains(&usage.span)
    }) {
        for method in &implementation.syntax.items {
            if let syn::ImplItem::Fn(method) = method
                && output_contains(&method.sig.output, usage)
            {
                return Some(ReturnedCallable {
                    identity: owner.1,
                    member: Some(method.sig.ident.to_string()),
                    producer: Some(owner),
                });
            }
        }
    }
    None
}

pub(crate) fn returned_to_other_owner(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> bool {
    context
        .name_index()
        .users_of(ordinal, identity)
        .filter(|usage| usage.kind == WorkspaceUseKind::Type && !usage.is_test && !usage.ambiguous)
        .any(|usage| {
            let Some(callable) = callable(context, usage) else {
                return false;
            };
            context
                .name_index()
                .users_of(usage.source_index, callable.identity)
                .filter(|call| {
                    call.kind == WorkspaceUseKind::Call
                        && !call.is_test
                        && !call.ambiguous
                        && usage.cfg.and(&call.cfg).is_satisfiable()
                })
                .filter(|call| {
                    callable
                        .member
                        .as_ref()
                        .is_none_or(|name| call.path.last() == Some(name))
                })
                .any(|call| {
                    let Some(consumer) = declaration_owner(context, call) else {
                        return false;
                    };
                    consumer != (ordinal, identity) && Some(consumer) != callable.producer
                })
        })
}
