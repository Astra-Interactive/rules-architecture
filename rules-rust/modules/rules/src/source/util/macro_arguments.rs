use crate::source::model::SelectPart;
use crate::source::util::parse_select_arguments;
use syn::Expr;
use syn::Macro;
use syn::Token;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

pub(crate) fn parse_macro_arguments(item: &Macro) -> Vec<Expr> {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    match parser.parse2(item.tokens.clone()) {
        Ok(arguments) => arguments.into_iter().collect(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn parse_select_parts(item: &Macro) -> Vec<SelectPart> {
    let is_select = item
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "select" || segment.ident == "select_biased");
    if !is_select {
        return Vec::new();
    }
    parse_select_arguments
        .parse2(item.tokens.clone())
        .unwrap_or_default()
}

pub(crate) fn parse_macro_expressions(item: &Macro) -> Vec<Expr> {
    let arguments = parse_macro_arguments(item);
    if !arguments.is_empty() {
        return arguments;
    }
    parse_select_parts(item)
        .into_iter()
        .filter_map(|part| match part {
            SelectPart::Expression(expression) => Some(expression),
            SelectPart::Pattern(_) => None,
        })
        .collect()
}
