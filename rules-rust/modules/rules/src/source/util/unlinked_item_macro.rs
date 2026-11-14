use std::collections::BTreeSet;
use syn::Token;
use syn::ext::IdentExt;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

pub(crate) fn is_unlinked_item_macro(
    item: &syn::ItemMacro,
    external_crates: &BTreeSet<String>,
) -> bool {
    let segments = &item.mac.path.segments;
    let (Some(first), Some(last)) = (segments.first(), segments.last()) else {
        return false;
    };
    let is_path = segments.len() > 1;
    let is_declaration_or_load =
        ["macro_rules", "include"].contains(&last.ident.to_string().as_str());
    let is_external = external_crates.contains(&first.ident.unraw().to_string());
    if !is_path || is_declaration_or_load || !is_external {
        return false;
    }
    Punctuated::<syn::Ident, Token![,]>::parse_terminated
        .parse2(item.mac.tokens.clone())
        .is_ok()
}
