use syn::punctuated::Punctuated;

pub(crate) fn list_derive_paths(attributes: &[syn::Attribute]) -> Vec<syn::Path> {
    attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("derive"))
        .filter_map(|attribute| {
            attribute
                .parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                .ok()
        })
        .flatten()
        .collect()
}
