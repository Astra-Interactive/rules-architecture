use crate::check::model::RuleContext;
use crate::module::util::find_type_path;
use crate::source::model::IndexedItem;
use syn::{FnArg, Item, ReturnType, TraitItem, Type};

impl RuleContext {
    fn registration_parameter_external(
        &self,
        origin: usize,
        ordinal: usize,
        module: usize,
        typed: &Type,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        self.resolve_workspace_type(ordinal, module, typed)
            .iter()
            .any(|target| target.index != origin && cfg.and(&target.path.cfg).is_satisfiable())
            || !self
                .resolve_outer_external_type_paths_under(ordinal, module, typed, cfg)
                .is_empty()
    }

    fn has_external_registration_parameter(
        &self,
        origin: usize,
        ordinal: usize,
        module: usize,
        typed: &Type,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        self.resolve_type_syntax_under(ordinal, module, typed, cfg)
            .iter()
            .all(|resolved| {
                if resolved.has_unsubstituted_parameters {
                    return false;
                }
                if let Type::Reference(reference) = &resolved.typed {
                    return self.has_external_registration_parameter(
                        origin,
                        resolved.index,
                        resolved.module,
                        &reference.elem,
                        cfg,
                    );
                }
                let Some(path) = find_type_path(&resolved.typed) else {
                    return false;
                };
                if !self.registration_parameter_external(
                    origin,
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                    cfg,
                ) {
                    return false;
                }
                path.segments
                    .iter()
                    .all(|segment| match &segment.arguments {
                        syn::PathArguments::None => true,
                        syn::PathArguments::AngleBracketed(arguments) => {
                            arguments.args.iter().all(|argument| match argument {
                                syn::GenericArgument::Type(typed) => self
                                    .has_external_registration_parameter(
                                        origin,
                                        resolved.index,
                                        resolved.module,
                                        typed,
                                        cfg,
                                    ),
                                syn::GenericArgument::AssocType(binding) => self
                                    .has_external_registration_parameter(
                                        origin,
                                        resolved.index,
                                        resolved.module,
                                        &binding.ty,
                                        cfg,
                                    ),
                                _ => true,
                            })
                        }
                        _ => false,
                    })
            })
    }

    pub(crate) fn has_registration_shape(&self, ordinal: usize, item: &IndexedItem) -> bool {
        let Item::Trait(contract) = &item.syntax else {
            return false;
        };
        !contract.items.is_empty() && contract.items.iter().all(|member| {
            let TraitItem::Fn(method) = member else { return false; };
            let returns_unit = matches!(method.sig.output, ReturnType::Default) || matches!(&method.sig.output, ReturnType::Type(_, typed) if matches!(typed.as_ref(), Type::Tuple(tuple) if tuple.elems.is_empty()));
            let receiver = matches!(method.sig.inputs.first(), Some(FnArg::Receiver(receiver)) if matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, None)) && receiver.mutability.is_none());
            method.sig.asyncness.is_none() && method.default.is_none() && returns_unit && receiver && method.sig.inputs.iter().skip(1).all(|parameter| matches!(parameter, FnArg::Typed(parameter) if self.has_external_registration_parameter(ordinal, ordinal, item.module, &parameter.ty, &item.cfg.and(&crate::source::model::CfgPredicate::from_attributes(&method.attrs)))))
        })
    }
}
