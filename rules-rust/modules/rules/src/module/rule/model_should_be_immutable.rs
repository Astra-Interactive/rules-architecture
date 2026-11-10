use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::is_model_item;
use crate::source::model::CfgPredicate;
use crate::source::model::IndexedImpl;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;

pub(crate) struct ModelShouldBeImmutable;

impl ModelShouldBeImmutable {
    fn assignments(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let mut findings = Vec::new();
        for item in &index.items {
            if !item.cfg.is_satisfiable() {
                continue;
            }
            if let syn::Item::Fn(function) = &item.syntax {
                findings.extend(crate::module::rule::ModelMutationVisitor::inspect(
                    context,
                    ordinal,
                    item.module,
                    &item.file,
                    &function.sig,
                    &function.block,
                    None,
                ));
            }
        }
        for implementation in &index.implementations {
            for member in &implementation.syntax.items {
                let syn::ImplItem::Fn(method) = member else {
                    continue;
                };
                if !implementation
                    .cfg
                    .and(&CfgPredicate::from_attributes(&method.attrs))
                    .is_satisfiable()
                {
                    continue;
                }
                findings.extend(crate::module::rule::ModelMutationVisitor::inspect(
                    context,
                    ordinal,
                    implementation.module,
                    &implementation.file,
                    &method.sig,
                    &method.block,
                    Some(&implementation.syntax.self_ty),
                ));
            }
        }
        findings
    }

    fn is_mutable_receiver_type(receiver_type: &syn::Type) -> bool {
        match receiver_type {
            syn::Type::Reference(reference) => reference.mutability.is_some(),
            syn::Type::Path(path) => path.path.segments.last().is_some_and(|segment| {
                let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return false;
                };
                segment.ident == "Pin"
                    && arguments.args.iter().any(|argument| match argument {
                        syn::GenericArgument::Type(inner) => Self::is_mutable_receiver_type(inner),
                        _ => false,
                    })
            }),
            _ => false,
        }
    }

    fn check_impl(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<RuleFinding> {
        let implementation_cfg = implementation.cfg.in_scope(ordinal);
        let models = context
            .resolve_outer_workspace_type(
                ordinal,
                implementation.module,
                &implementation.syntax.self_ty,
            )
            .iter()
            .filter(|target| {
                target.path.item.is_some_and(|identity| {
                    is_model_item(
                        context,
                        target.index,
                        &context.indexes[target.index].items[identity],
                    )
                })
            })
            .map(|target| target.path.cfg.and(&implementation_cfg))
            .filter(CfgPredicate::is_satisfiable)
            .collect::<Vec<_>>();
        if models.is_empty() {
            return Vec::new();
        }
        implementation.syntax.items.iter().filter_map(|item| {
            let syn::ImplItem::Fn(method) = item else { return None; };
            let method_cfg = CfgPredicate::from_attributes(&method.attrs).in_scope(ordinal);
            if !models.iter().any(|model| model.and(&method_cfg).is_satisfiable()) { return None; }
            let mutable_receiver = method.sig.inputs.iter().any(|argument| matches!(argument, syn::FnArg::Receiver(receiver) if matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, Some(_))) || matches!(&receiver.kind, syn::ReceiverKind::Typed(_, value) if Self::is_mutable_receiver_type(value))));
            mutable_receiver.then(|| build_node_finding("ModelShouldBeImmutable", &implementation.file, find_node_span(method.span()), method.sig.ident.to_string(), "A model transition returns a new value instead of taking &mut self (arch-domain.rs.md)".to_owned()))
        }).collect()
    }
}

impl Rule for ModelShouldBeImmutable {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                Self::assignments(context, ordinal).into_iter().chain(
                    index
                        .implementations
                        .iter()
                        .flat_map(move |implementation| {
                            Self::check_impl(context, ordinal, implementation)
                        }),
                )
            })
            .collect()
    }
}
