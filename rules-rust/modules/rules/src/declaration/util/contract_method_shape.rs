use crate::check::model::RuleContext;
use crate::source::model::IndexedItem;

fn closure_type(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    context
        .resolve_type_syntax(ordinal, module, typed)
        .iter()
        .filter(|resolved| cfg.and(&resolved.cfg).is_satisfiable())
        .any(|resolved| matches!(resolved.typed, syn::Type::FnPtr(_)))
        || context
            .resolve_external_type_paths_under(ordinal, module, typed, cfg)
            .iter()
            .any(|path| {
                ["std", "core"].contains(&path.package.as_str())
                    && path
                        .segments
                        .iter()
                        .any(|name| ["Fn", "FnMut", "FnOnce"].contains(&name.as_str()))
            })
}

fn event_stream(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    context.resolve_type_syntax(ordinal, module, typed).iter().filter(|resolved| cfg.and(&resolved.cfg).is_satisfiable()).all(|resolved| {
        if resolved.has_unsubstituted_parameters { return true; }
        let outer = context.resolve_outer_external_type_paths_under(resolved.index, resolved.module, &resolved.typed, cfg);
        let stream = outer.iter().any(|path| ["futures", "futures_util", "futures_core"].contains(&path.package.as_str()) && path.segments.last().is_some_and(|segment| segment == "BoxStream"));
        let syn::Type::Path(path) = &resolved.typed else { return false; };
        let Some(segment) = path.path.segments.last() else { return false; };
        stream && matches!(&segment.arguments, syn::PathArguments::AngleBracketed(arguments) if matches!(arguments.args.first(), Some(syn::GenericArgument::Lifetime(lifetime)) if lifetime.ident == "static"))
    })
}

fn invalid_return(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    if closure_type(context, ordinal, module, typed, cfg) {
        return true;
    }
    let paths = context.resolve_external_type_paths_under(ordinal, module, typed, cfg);
    let handle = paths.iter().any(|path| {
        ["tokio", "std", "futures", "futures_channel"].contains(&path.package.as_str())
            && (path
                .segments
                .last()
                .is_some_and(|name| name == "JoinHandle")
                || path
                    .segments
                    .ends_with(&["oneshot".to_owned(), "Receiver".to_owned()])
                || (path.package == "tokio"
                    && [
                        "broadcast::Receiver",
                        "mpsc::Receiver",
                        "mpsc::UnboundedReceiver",
                    ]
                    .iter()
                    .any(|channel| {
                        let names: Vec<_> = channel.split("::").map(str::to_owned).collect();
                        path.segments.ends_with(&names)
                    })))
    });
    let stream = paths.iter().any(|path| {
        ["futures", "futures_core", "futures_util"].contains(&path.package.as_str())
            && path.segments.last().is_some_and(|name| name == "Stream")
    });
    let raw_stream = stream
        && context
            .resolve_type_syntax(ordinal, module, typed)
            .iter()
            .filter(|resolved| cfg.and(&resolved.cfg).is_satisfiable())
            .any(|resolved| {
                matches!(resolved.typed, syn::Type::ImplTrait(_))
                    || context
                        .resolve_outer_external_type_paths_under(
                            resolved.index,
                            resolved.module,
                            &resolved.typed,
                            cfg,
                        )
                        .iter()
                        .any(|path| {
                            ["std", "core"].contains(&path.package.as_str())
                                && path.segments.last().is_some_and(|name| name == "Pin")
                        })
            });
    handle || raw_stream || external_result_error(context, ordinal, module, typed, cfg)
}

fn external_result_error(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    context
        .resolve_type_syntax(ordinal, module, typed)
        .iter()
        .filter(|resolved| cfg.and(&resolved.cfg).is_satisfiable())
        .any(|resolved| {
            if resolved.has_unsubstituted_parameters {
                return false;
            }
            let result = context
                .resolve_outer_external_type_paths_under(
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                    cfg,
                )
                .iter()
                .any(|path| {
                    ["std", "core"].contains(&path.package.as_str())
                        && path.segments.last().is_some_and(|name| name == "Result")
                });
            let syn::Type::Path(path) = &resolved.typed else {
                return false;
            };
            let Some(segment) = path.path.segments.last() else {
                return false;
            };
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return false;
            };
            let Some(syn::GenericArgument::Type(error)) = arguments.args.iter().nth(1) else {
                return false;
            };
            result
                && !context
                    .resolve_outer_external_type_paths_under(
                        resolved.index,
                        resolved.module,
                        error,
                        cfg,
                    )
                    .is_empty()
        })
}

fn signature_uses(signature: &syn::Signature, name: &syn::Ident) -> bool {
    let mut types: Vec<_> = signature
        .inputs
        .iter()
        .filter_map(|input| match input {
            syn::FnArg::Typed(parameter) => Some(parameter.ty.as_ref()),
            _ => None,
        })
        .collect();
    if let syn::ReturnType::Type(_, typed) = &signature.output {
        types.push(typed);
    }
    types
        .into_iter()
        .flat_map(crate::module::rule::PlacementTypeVisitor::collect_paths)
        .any(|path| path.is_ident(name))
}

fn signature_closure(
    context: &RuleContext,
    ordinal: usize,
    item: &IndexedItem,
    signature: &syn::Signature,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let mut bounds: Vec<_> = signature
        .generics
        .type_params()
        .filter(|parameter| signature_uses(signature, &parameter.ident))
        .map(|parameter| &parameter.bounds)
        .collect();
    if let Some(clause) = &signature.generics.where_clause {
        bounds.extend(clause.predicates.iter().filter_map(|predicate| {
            let syn::WherePredicate::Type(predicate) = predicate else {
                return None;
            };
            let syn::Type::Path(path) = &predicate.bounded_ty else {
                return None;
            };
            path.path
                .get_ident()
                .filter(|name| signature_uses(signature, name))
                .map(|_| &predicate.bounds)
        }));
    }
    bounds
        .into_iter()
        .filter(|bounds| !bounds.is_empty())
        .any(|bounds| {
            let typed: syn::Type = syn::parse_quote!(impl #bounds);
            closure_type(context, ordinal, item.module, &typed, cfg)
        })
}

pub(crate) fn invalid_contract_method(
    context: &RuleContext,
    ordinal: usize,
    item: &IndexedItem,
    method: &syn::TraitItemFn,
) -> bool {
    let cfg = &item
        .cfg
        .and(&crate::source::model::CfgPredicate::from_attributes(
            &method.attrs,
        ));
    if !cfg.is_satisfiable() {
        return false;
    }
    let callback = method.sig.inputs.iter().any(|input| matches!(input, syn::FnArg::Typed(parameter) if closure_type(context, ordinal, item.module, &parameter.ty, cfg))) || signature_closure(context, ordinal, item, &method.sig, cfg);
    let output = match &method.sig.output {
        syn::ReturnType::Type(_, typed) => Some(typed.as_ref()),
        _ => None,
    };
    let events = method.sig.ident == "events"
        && output.is_none_or(|typed| !event_stream(context, ordinal, item.module, typed, cfg));
    callback
        || events
        || output.is_some_and(|typed| invalid_return(context, ordinal, item.module, typed, cfg))
}
