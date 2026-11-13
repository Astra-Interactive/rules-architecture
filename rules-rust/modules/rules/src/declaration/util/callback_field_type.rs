use crate::check::model::RuleContext;
use crate::source::model::{CfgPredicate, Resolution};

fn listener_bound(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    path: &syn::Path,
    cfg: &CfgPredicate,
) -> bool {
    let typed: syn::Type = syn::parse_quote!(dyn #path);
    let closure = context
        .resolve_external_type_paths_under(ordinal, module, &typed, cfg)
        .iter()
        .any(|path| {
            ["std", "core"].contains(&path.package.as_str())
                && path
                    .segments
                    .last()
                    .is_some_and(|name| ["Fn", "FnMut", "FnOnce"].contains(&name.as_str()))
        });
    if closure {
        return true;
    }
    let targets = context.resolve_workspace_path(ordinal, module, path);
    if !targets.is_empty() {
        return targets.iter().any(|target| {
            target.path.item.is_some_and(|identity| {
                let item = &context.indexes[target.index].items[identity];
                matches!(item.syntax, syn::Item::Trait(_))
                    && (item.name.ends_with("Listener") || item.name.ends_with("Callback"))
            })
        });
    }
    let Resolution::External(path) = context.indexes[ordinal].resolve_path(module, path) else {
        return false;
    };
    let resolved = context.resolve_external_identity(ordinal, &path);
    resolved.segments.last().is_some_and(|name| {
        name.ends_with("Listener")
            || name.ends_with("Callback")
            || (["std", "core"].contains(&resolved.package.as_str())
                && ["Fn", "FnMut", "FnOnce"].contains(&name.as_str()))
    })
}

fn generic_closure(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    generics: &syn::Generics,
    name: &syn::Ident,
    cfg: &CfgPredicate,
) -> bool {
    let mut bounds: Vec<_> = generics
        .type_params()
        .filter(|parameter| parameter.ident == *name)
        .map(|parameter| &parameter.bounds)
        .collect();
    if let Some(clause) = &generics.where_clause {
        bounds.extend(clause.predicates.iter().filter_map(|predicate| match predicate { syn::WherePredicate::Type(predicate) if matches!(&predicate.bounded_ty, syn::Type::Path(path) if path.path.is_ident(name)) => Some(&predicate.bounds), _ => None }));
    }
    bounds
        .into_iter()
        .filter(|bounds| !bounds.is_empty())
        .any(|bounds| {
            let typed: syn::Type = syn::parse_quote!(impl #bounds);
            context
                .resolve_external_type_paths_under(ordinal, module, &typed, cfg)
                .iter()
                .any(|path| {
                    ["std", "core"].contains(&path.package.as_str())
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["Fn", "FnMut", "FnOnce"].contains(&name.as_str()))
                })
        })
}

fn is_single_value_wrapper(path: &crate::source::model::ExternalPath) -> bool {
    let name = path.segments.last().map_or("", String::as_str);
    let standard = ["std", "core", "alloc"].contains(&path.package.as_str());
    let library = ["tokio", "parking_lot", "once_cell"].contains(&path.package.as_str());
    let pointer = ["Option", "Box", "Arc", "Rc", "Pin"].contains(&name);
    let cell = ["Mutex", "RwLock", "OnceLock", "Cell", "RefCell", "OnceCell"].contains(&name);
    (standard && pointer) || ((standard || library) && cell)
}

fn syntax_callback(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    cfg: &CfgPredicate,
    generics: &syn::Generics,
    depth: usize,
) -> bool {
    match typed {
        syn::Type::FnPtr(_) => true,
        syn::Type::Reference(reference) => callback_field_type(context, ordinal, module, &reference.elem, cfg, generics, depth + 1),
        syn::Type::Ptr(pointer) => callback_field_type(context, ordinal, module, &pointer.elem, cfg, generics, depth + 1),
        syn::Type::TraitObject(object) => object.bounds.iter().any(|bound| matches!(bound, syn::TypeParamBound::Trait(bound) if listener_bound(context, ordinal, module, &bound.path, cfg))),
        syn::Type::Path(path) => {
            if path.path.get_ident().is_some_and(|name| generic_closure(context, ordinal, module, generics, name, cfg)) { return true; }
            let pointer = context.resolve_outer_external_type_paths_under(ordinal, module, typed, cfg).iter().any(is_single_value_wrapper);
            let Some(segment) = path.path.segments.last() else { return false; };
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else { return false; };
            pointer && arguments.args.iter().any(|argument| matches!(argument, syn::GenericArgument::Type(typed) if callback_field_type(context, ordinal, module, typed, cfg, generics, depth + 1)))
        }
        _ => false,
    }
}

pub(crate) fn callback_field_type(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    cfg: &CfgPredicate,
    generics: &syn::Generics,
    depth: usize,
) -> bool {
    if depth > 8 {
        return false;
    }
    context
        .resolve_type_syntax_under(ordinal, module, typed, cfg)
        .iter()
        .filter(|resolved| cfg.and(&resolved.cfg).is_satisfiable())
        .any(|resolved| {
            !resolved.has_unsubstituted_parameters
                && syntax_callback(
                    context,
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                    cfg,
                    generics,
                    depth,
                )
        })
}
