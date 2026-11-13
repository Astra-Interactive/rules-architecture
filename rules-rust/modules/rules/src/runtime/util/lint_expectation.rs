pub(crate) fn expects_lint(attributes: &[syn::Attribute], lint: &str) -> bool {
    attributes.iter().any(|attribute| {
        if !attribute.path().is_ident("expect") {
            return false;
        }
        let Ok(entries) = attribute.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        ) else {
            return false;
        };
        entries.iter().any(|entry| {
            matches!(entry, syn::Meta::Path(path)
            if path.segments.len() == 2 && path.segments[0].ident == "clippy"
            && path.segments[1].ident == lint)
        })
    })
}
