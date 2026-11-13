use crate::check::model::RuleContext;
use crate::comment::model::DocOwner;
use crate::module::util::owns_implementation;

fn item_member(context: &RuleContext, ordinal: usize, identity: usize, name: &str) -> bool {
    let index = &context.indexes[ordinal];
    let item = &index.items[identity];
    let own = match &item.syntax {
        syn::Item::Struct(value) => value
            .fields
            .iter()
            .any(|field| field.ident.as_ref().is_some_and(|field| field == name)),
        syn::Item::Union(value) => value
            .fields
            .named
            .iter()
            .any(|field| field.ident.as_ref().is_some_and(|field| field == name)),
        syn::Item::Enum(value) => value.variants.iter().any(|variant| variant.ident == name),
        syn::Item::Trait(value) => value.items.iter().any(|item| match item {
            syn::TraitItem::Fn(item) => item.sig.ident == name,
            syn::TraitItem::Type(item) => item.ident == name,
            syn::TraitItem::Const(item) => item.ident == name,
            _ => false,
        }),
        _ => false,
    };
    own || index
        .implementations
        .iter()
        .filter(|implementation| owns_implementation(index, identity, implementation))
        .any(|implementation| {
            implementation.syntax.items.iter().any(|item| match item {
                syn::ImplItem::Fn(item) => item.sig.ident == name,
                syn::ImplItem::Const(item) => item.ident == name,
                syn::ImplItem::Type(item) => item.ident == name,
                _ => false,
            })
        })
}

fn path_resolves(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    owner: Option<(usize, usize)>,
    candidate: &str,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let Ok(path) = syn::parse_str::<syn::Path>(candidate) else {
        return false;
    };
    let Some(last) = path.segments.last() else {
        return false;
    };
    let targets = context.resolve_workspace_path(ordinal, module, &path);
    if path.segments.len() == 1
        && targets
            .iter()
            .any(|target| target.path.item.is_some() && cfg.and(&target.path.cfg).is_satisfiable())
    {
        return true;
    }
    if targets
        .iter()
        .filter(|target| cfg.and(&target.path.cfg).is_satisfiable())
        .any(|target| {
            target.path.item.is_some_and(|identity| {
                last.ident == context.indexes[target.index].items[identity].name
            })
        })
    {
        return true;
    }
    if path.is_ident("Self") {
        return owner.is_some();
    }
    if path.segments.len() == 2 {
        let mut parent = path.clone();
        parent.segments.pop();
        parent.segments.pop_punct();
        let module_parent = context
            .resolve_workspace_path(ordinal, module, &parent)
            .iter()
            .any(|target| target.path.item.is_none() && cfg.and(&target.path.cfg).is_satisfiable());
        if module_parent
            && targets.iter().any(|target| {
                target.path.item.is_some() && cfg.and(&target.path.cfg).is_satisfiable()
            })
        {
            return true;
        }
        let owner = if parent.is_ident("Self") {
            owner.into_iter().collect()
        } else {
            parent_types(context, ordinal, module, &parent, cfg)
        };
        return owner.into_iter().any(|(ordinal, identity)| {
            item_member(context, ordinal, identity, &last.ident.to_string())
        });
    }
    if path.segments.len() != 1 {
        return false;
    }
    prelude_resolves(context, ordinal, module, &path, cfg)
}

fn parent_types(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    path: &syn::Path,
    cfg: &crate::source::model::CfgPredicate,
) -> Vec<(usize, usize)> {
    let typed: syn::Type = syn::parse_quote!(#path);
    context
        .resolve_outer_workspace_type(ordinal, module, &typed)
        .iter()
        .filter(|target| cfg.and(&target.path.cfg).is_satisfiable())
        .filter_map(|target| target.path.item.map(|identity| (target.index, identity)))
        .collect()
}

fn prelude_resolves(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    path: &syn::Path,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let Some(last) = path.segments.last() else {
        return false;
    };
    let name = last.ident.to_string();
    if [
        "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32",
        "i64", "i128", "isize", "f32", "f64",
    ]
    .contains(&name.as_str())
    {
        return true;
    }
    let typed: syn::Type = syn::parse_quote!(#path);
    context
        .resolve_outer_external_type_paths_under(ordinal, module, &typed, cfg)
        .iter()
        .any(|path| {
            ["std", "core", "alloc"].contains(&path.package.as_str())
                && path.segments.last().is_some_and(|name| {
                    [
                        "String", "Vec", "Option", "Result", "Box", "Arc", "Rc", "Send", "Sync",
                    ]
                    .contains(&name.as_str())
                })
        })
}

fn has_function_member(context: &RuleContext, owner: &DocOwner, name: &str) -> bool {
    let index = &context.indexes[owner.ordinal];
    let identity = owner.identity;
    let in_trait = matches!(
        &index.items[identity].syntax,
        syn::Item::Trait(value) if value.items.iter().any(|item| {
            matches!(item, syn::TraitItem::Fn(function) if function.sig.ident == name)
        })
    );
    in_trait
        || index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
            .any(|implementation| {
                implementation.syntax.items.iter().any(|item| {
                    matches!(item, syn::ImplItem::Fn(function) if function.sig.ident == name)
                })
            })
}

fn function_resolves(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    owner: Option<DocOwner>,
    name: &str,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let Ok(path) = syn::parse_str::<syn::Path>(name) else {
        return false;
    };
    let Some(last) = path.segments.last() else {
        return false;
    };
    let is_function = |target: &crate::source::model::WorkspaceResolvedPath| {
        cfg.and(&target.path.cfg).is_satisfiable()
            && target.path.item.is_some_and(|identity| {
                let item = &context.indexes[target.index].items[identity];
                matches!(item.syntax, syn::Item::Fn(_))
                    && (path.segments.len() == 1 || last.ident == item.name)
            })
    };
    if context
        .resolve_workspace_path(ordinal, module, &path)
        .iter()
        .any(is_function)
    {
        return true;
    }
    if path.segments.len() != 2 {
        return false;
    }
    let mut parent = path.clone();
    parent.segments.pop();
    parent.segments.pop_punct();
    let owners: Vec<DocOwner> = if parent.is_ident("Self") {
        owner.into_iter().collect()
    } else {
        parent_types(context, ordinal, module, &parent, cfg)
            .into_iter()
            .map(|(ordinal, identity)| DocOwner { ordinal, identity })
            .collect()
    };
    owners
        .iter()
        .any(|owner| has_function_member(context, owner, &last.ident.to_string()))
}

pub(crate) fn doc_resolves(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    owner: Option<(usize, usize)>,
    candidate: &str,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    match candidate.strip_suffix("()") {
        Some(name) => {
            let owner = owner.map(|(ordinal, identity)| DocOwner { ordinal, identity });
            function_resolves(context, ordinal, module, owner, name, cfg)
        }
        None => path_resolves(context, ordinal, module, owner, candidate, cfg),
    }
}
