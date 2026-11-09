use crate::check::model::RuleContext;
use crate::declaration::util::derives;
use crate::source::model::Resolution;

fn item_supports(
    context: &RuleContext,
    target: &crate::source::model::WorkspaceResolvedPath,
) -> bool {
    let Some(identity) = target.path.item else {
        return false;
    };
    let item = &context.indexes[target.index].items[identity];
    let attributes = match &item.syntax {
        syn::Item::Struct(value) => &value.attrs,
        syn::Item::Enum(value) => &value.attrs,
        _ => return false,
    };
    derives(attributes, "Eq") && derives(attributes, "Hash")
}

fn primitive_eq_hash(name: &str) -> bool {
    [
        "bool", "char", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
        "i128", "isize", "str",
    ]
    .contains(&name)
}

fn path_supports(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    path: &syn::TypePath,
    depth: usize,
) -> bool {
    let targets = context.resolve_workspace_path(ordinal, module, &path.path);
    if !targets.is_empty() {
        return targets.iter().all(|target| item_supports(context, target));
    }
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let name = segment.ident.to_string();
    let primitive = primitive_eq_hash(&name);
    if primitive
        && matches!(
            context.indexes[ordinal].resolve_path(module, &path.path),
            Resolution::Local
        )
    {
        return true;
    }
    let typed = syn::Type::Path(path.clone());
    let paths = context.resolve_outer_external_type_paths(ordinal, module, &typed);
    let Some(path) = paths
        .iter()
        .find(|path| ["std", "core", "alloc"].contains(&path.package.as_str()))
    else {
        return false;
    };
    let Some(name) = path.segments.last() else {
        return false;
    };
    if [
        "String",
        "PathBuf",
        "Path",
        "OsString",
        "OsStr",
        "Duration",
        "SystemTime",
        "Instant",
    ]
    .contains(&name.as_str())
    {
        return true;
    }
    if !["Option", "Result", "Vec", "Box", "Arc", "Rc"].contains(&name.as_str()) {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    arguments
        .args
        .iter()
        .filter_map(|argument| {
            if let syn::GenericArgument::Type(typed) = argument {
                Some(typed)
            } else {
                None
            }
        })
        .all(|typed| supports_eq_hash(context, ordinal, module, typed, depth + 1))
}

fn syntax_supports(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    depth: usize,
) -> bool {
    match typed {
        syn::Type::Reference(reference) => {
            supports_eq_hash(context, ordinal, module, &reference.elem, depth + 1)
        }
        syn::Type::Tuple(tuple) => tuple
            .elems
            .iter()
            .all(|typed| supports_eq_hash(context, ordinal, module, typed, depth + 1)),
        syn::Type::Array(array) => {
            supports_eq_hash(context, ordinal, module, &array.elem, depth + 1)
        }
        syn::Type::Slice(slice) => {
            supports_eq_hash(context, ordinal, module, &slice.elem, depth + 1)
        }
        syn::Type::Path(path) => path_supports(context, ordinal, module, path, depth),
        _ => false,
    }
}

pub(crate) fn supports_eq_hash(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    typed: &syn::Type,
    depth: usize,
) -> bool {
    if depth > 8 {
        return false;
    }
    context
        .resolve_type_syntax(ordinal, module, typed)
        .iter()
        .filter(|resolved| resolved.cfg.is_satisfiable())
        .all(|resolved| {
            !resolved.has_unsubstituted_parameters
                && syntax_supports(
                    context,
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                    depth,
                )
        })
}
