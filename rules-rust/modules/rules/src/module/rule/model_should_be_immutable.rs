use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::{is_model_item, owns_implementation};
use crate::source::model::CfgPredicate;
use crate::source::model::IndexedImpl;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;

pub(crate) struct ModelShouldBeImmutable;

impl ModelShouldBeImmutable {
    fn check_impl(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let models = index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .iter()
            .filter(|identity| {
                is_model_item(context, ordinal, &index.items[**identity])
                    && owns_implementation(index, **identity, implementation)
            })
            .map(|identity| index.items[*identity].cfg.and(&implementation.cfg))
            .collect::<Vec<_>>();
        if models.is_empty() {
            return Vec::new();
        }
        implementation.syntax.items.iter().filter_map(|item| {
            let syn::ImplItem::Fn(method) = item else { return None; };
            let method_cfg = CfgPredicate::from_attributes(&method.attrs);
            if !models.iter().any(|model| model.and(&method_cfg).is_satisfiable()) { return None; }
            let mutable_receiver = method.sig.inputs.iter().any(|argument| matches!(argument, syn::FnArg::Receiver(receiver) if matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, Some(_))) || matches!(&receiver.kind, syn::ReceiverKind::Typed(_, value) if matches!(value.as_ref(), syn::Type::Reference(reference) if reference.mutability.is_some()))));
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
                index
                    .implementations
                    .iter()
                    .flat_map(move |implementation| {
                        Self::check_impl(context, ordinal, implementation)
                    })
            })
            .collect()
    }
}
