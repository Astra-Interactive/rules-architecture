use crate::check::model::RuleContext;

fn is_collection(context: &RuleContext, ordinal: usize, module: usize, typed: &syn::Type) -> bool {
    context
        .resolve_outer_external_type_paths(ordinal, module, typed)
        .iter()
        .any(|path| {
            matches!(path.package.as_str(), "std" | "alloc" | "core")
                && path.segments.last().is_some_and(|name| {
                    [
                        "Vec", "VecDeque", "HashMap", "BTreeMap", "HashSet", "BTreeSet", "Option",
                    ]
                    .contains(&name.as_str())
                })
        })
}

fn data_fields(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
) -> Vec<String> {
    if is_collection(context, ordinal, module, typed) {
        return vec![String::new()];
    }
    context
        .resolve_workspace_type(ordinal, module, typed)
        .iter()
        .flat_map(|target| {
            let Some(identity) = target.path.item else {
                return Vec::new();
            };
            let declaration = &context.indexes[target.index].items[identity];
            let syn::Item::Struct(item) = &declaration.syntax else {
                return Vec::new();
            };
            item.fields
                .iter()
                .enumerate()
                .filter(|(_, field)| {
                    is_collection(context, target.index, declaration.module, &field.ty)
                })
                .map(|(position, field)| {
                    field
                        .ident
                        .as_ref()
                        .map_or_else(|| position.to_string(), ToString::to_string)
                })
                .collect()
        })
        .collect()
}

pub(crate) fn guard_collection_fields(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    depth: usize,
) -> Vec<String> {
    if depth > 8 {
        return Vec::new();
    }
    if let syn::Type::Reference(reference) = typed {
        return guard_collection_fields(context, ordinal, module, &reference.elem, depth + 1);
    }
    context
        .resolve_type_syntax(ordinal, module, typed)
        .iter()
        .flat_map(|resolved| {
            let syn::Type::Path(path) = &resolved.typed else {
                return Vec::new();
            };
            let Some(segment) = path.path.segments.last() else {
                return Vec::new();
            };
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return Vec::new();
            };
            let Some(syn::GenericArgument::Type(inner)) = arguments.args.first() else {
                return Vec::new();
            };
            let identities = context.resolve_outer_external_type_paths(
                resolved.index,
                resolved.module,
                &resolved.typed,
            );
            if identities.iter().any(|path| {
                path.segments
                    .last()
                    .is_some_and(|name| matches!(name.as_str(), "Mutex" | "RwLock"))
            }) {
                return data_fields(context, resolved.index, resolved.module, inner);
            }
            if identities.iter().any(|path| {
                matches!(path.package.as_str(), "std" | "alloc" | "core")
                    && path
                        .segments
                        .last()
                        .is_some_and(|name| matches!(name.as_str(), "Arc" | "Rc" | "Box" | "Pin"))
            }) {
                return guard_collection_fields(
                    context,
                    resolved.index,
                    resolved.module,
                    inner,
                    depth + 1,
                );
            }
            Vec::new()
        })
        .collect()
}
