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
                || index
                    .crate_name
                    .split('-')
                    .chain(index.modules[module].module.path.iter().map(String::as_str))
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
