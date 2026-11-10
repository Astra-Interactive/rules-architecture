use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::source::model::CrateIndex;
use syn::ext::IdentExt;

pub(crate) fn derives(attributes: &[syn::Attribute], name: &str) -> bool {
    attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("derive"))
        .any(|attribute| {
            attribute
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|paths| {
                    paths.iter().any(|path| {
                        path.segments
                            .last()
                            .is_some_and(|segment| segment.ident == name)
                    })
                })
        })
}

pub(crate) fn is_technology(context: &RuleContext, index: &CrateIndex, module: usize) -> bool {
    context
        .project_config
        .crates
        .iter()
        .find(|config| config.name == index.crate_name)
        .is_some_and(|config| {
            config.role == Some(Role::Technology)
                || index.modules[module]
                    .module
                    .path
                    .iter()
                    .any(|segment| config.technology_words.iter().any(|word| word == segment))
        })
}

pub(crate) fn has_kind(index: &CrateIndex, module: usize, kind: &str) -> bool {
    index.modules[module]
        .module
        .path
        .iter()
        .any(|segment| segment == kind)
}

pub(crate) fn has_bound(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    name: &str,
) -> bool {
    bounds.iter().any(|bound| matches!(bound, syn::TypeParamBound::Trait(bound) if bound.path.segments.last().is_some_and(|segment| segment.ident.unraw() == name)))
}

pub(crate) fn has_standard_bound(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    generics: &syn::Generics,
    typed: &syn::Type,
    name: &str,
) -> bool {
    let syn::Type::Path(typed) = typed else {
        return false;
    };
    let Some(parameter) = typed.path.get_ident() else {
        return false;
    };
    let Some(declaration) = generics
        .type_params()
        .find(|entry| entry.ident == *parameter)
    else {
        return false;
    };
    let mut bounds = declaration.bounds.iter().chain(
        generics.where_clause.iter().flat_map(|clause| {
            clause.predicates.iter().filter_map(|predicate| {
                let syn::WherePredicate::Type(predicate) = predicate else {
                    return None;
                };
                matches!(&predicate.bounded_ty, syn::Type::Path(path) if path.path.get_ident() == Some(parameter))
                    .then_some(&predicate.bounds)
            }).flatten()
        }),
    );
    bounds.any(|bound| {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return false;
        };
        if bound.maybe.is_some() || bound.modifiers.require_empty().is_err() {
            return false;
        }
        match context.indexes[ordinal].resolve_path(module, &bound.path) {
            crate::source::model::Resolution::Local => bound.path.is_ident(name),
            crate::source::model::Resolution::External(path) => {
                let namespace = match name {
                    "Copy" => "marker",
                    "Eq" => "cmp",
                    "Hash" => "hash",
                    _ => return false,
                };
                ["std", "core"]
                    .iter()
                    .any(|root| path == format!("{root}::{namespace}::{name}"))
            }
            _ => false,
        }
    })
}
