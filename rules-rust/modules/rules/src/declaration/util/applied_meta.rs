use syn::punctuated::Punctuated;
use syn::{Meta, Token};

fn flatten_cfg_attr(meta: &Meta) -> Vec<Meta> {
    let Meta::List(list) = meta else {
        return vec![meta.clone()];
    };
    if !list.path.is_ident("cfg_attr") {
        return vec![meta.clone()];
    }
    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .map(|metas| metas.iter().skip(1).flat_map(flatten_cfg_attr).collect())
        .unwrap_or_default()
}

pub(crate) fn list_applied_metas(attributes: &[syn::Attribute]) -> Vec<Meta> {
    attributes
        .iter()
        .flat_map(|attribute| flatten_cfg_attr(&attribute.meta))
        .collect()
}

pub(crate) fn derives_applied(metas: &[Meta], name: &str) -> bool {
    metas.iter().any(|meta| {
        let Meta::List(list) = meta else {
            return false;
        };
        list.path.is_ident("derive")
            && list
                .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
                .is_ok_and(|paths| {
                    paths.iter().any(|path| {
                        path.segments
                            .last()
                            .is_some_and(|segment| segment.ident == name)
                    })
                })
    })
}

pub(crate) fn has_serde_argument(metas: &[Meta], names: &[&str]) -> bool {
    metas.iter().any(|meta| {
        let Meta::List(list) = meta else {
            return false;
        };
        list.path.is_ident("serde")
            && list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .is_ok_and(|arguments| {
                    arguments
                        .iter()
                        .any(|argument| names.iter().any(|name| argument.path().is_ident(name)))
                })
    })
}
