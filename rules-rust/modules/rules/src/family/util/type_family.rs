use crate::source::model::CrateSources;
use crate::source::model::SourceFile;
use crate::syntax::util::find_type_name;
use crate::syntax::util::to_snake_case;
use proc_macro2::TokenTree;
use quote::ToTokens;
use std::path::Path;

pub(crate) fn find_primary_type<'syntax>(
    file: &'syntax SourceFile,
    sources: &CrateSources,
) -> Option<&'syntax syn::Item> {
    let stem = Path::new(&file.path).file_stem()?.to_str()?;
    let target = sources
        .modules
        .iter()
        .find(|module| module.id == file.module)
        .and_then(|module| module.declaration.as_ref())
        .and_then(|declaration| declaration.target_value.as_deref());
    let stem = target
        .and_then(|value| stem.strip_suffix(&format!("_{value}")))
        .unwrap_or(stem);
    file.syntax.items.iter().find(|item| {
        find_type_name(item).is_some_and(|name| to_snake_case(&name.to_string()) == stem)
    })
}

fn has_name(tokens: proc_macro2::TokenStream, name: &str) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(ident) => ident == name,
        TokenTree::Group(group) => has_name(group.stream(), name),
        _ => false,
    })
}

pub(crate) fn uses_type(item: &syn::Item, name: &str) -> bool {
    match item {
        syn::Item::Struct(value) => value
            .fields
            .iter()
            .any(|field| has_name(field.ty.to_token_stream(), name)),
        syn::Item::Enum(value) => value
            .variants
            .iter()
            .flat_map(|variant| &variant.fields)
            .any(|field| has_name(field.ty.to_token_stream(), name)),
        syn::Item::Union(value) => value
            .fields
            .named
            .iter()
            .any(|field| has_name(field.ty.to_token_stream(), name)),
        _ => false,
    }
}
