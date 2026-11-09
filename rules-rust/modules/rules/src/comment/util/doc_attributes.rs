use crate::source::model::NodeSpan;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;

pub(crate) fn joined_doc(attributes: &[syn::Attribute]) -> Option<(NodeSpan, String)> {
    let mut start = None;
    let mut lines = Vec::new();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
    {
        let syn::Meta::NameValue(value) = &attribute.meta else {
            continue;
        };
        let syn::Expr::Lit(value) = &value.value else {
            continue;
        };
        let syn::Lit::Str(text) = &value.lit else {
            continue;
        };
        start.get_or_insert_with(|| find_node_span(attribute.span()));
        lines.push(text.value());
    }
    start.map(|span| (span, lines.join("\n")))
}

pub(crate) fn doc_heading(text: &str, required: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with('#') && line.trim_start_matches('#').trim() == required
    })
}

pub(crate) fn item_doc_attributes(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(item) => &item.attrs,
        syn::Item::Enum(item) => &item.attrs,
        syn::Item::ExternCrate(item) => &item.attrs,
        syn::Item::Fn(item) => &item.attrs,
        syn::Item::ForeignMod(item) => &item.attrs,
        syn::Item::Impl(item) => &item.attrs,
        syn::Item::Macro(item) => &item.attrs,
        syn::Item::Mod(item) => &item.attrs,
        syn::Item::Static(item) => &item.attrs,
        syn::Item::Struct(item) => &item.attrs,
        syn::Item::Trait(item) => &item.attrs,
        syn::Item::TraitAlias(item) => &item.attrs,
        syn::Item::Type(item) => &item.attrs,
        syn::Item::Union(item) => &item.attrs,
        syn::Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

pub(crate) fn doc_attribute_lines(attributes: &[syn::Attribute]) -> Vec<NodeSpan> {
    let mut spans = Vec::new();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
    {
        let syn::Meta::NameValue(value) = &attribute.meta else {
            continue;
        };
        let syn::Expr::Lit(value) = &value.value else {
            continue;
        };
        let syn::Lit::Str(text) = &value.lit else {
            continue;
        };
        spans.extend(std::iter::repeat_n(
            find_node_span(attribute.span()),
            text.value().split('\n').count(),
        ));
    }
    spans
}
