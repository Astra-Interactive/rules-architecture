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

fn is_select_name(name: &str) -> bool {
    ["select", "select_biased"].contains(&name)
}

fn is_select_path(path: &syn::Path) -> bool {
    let names: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    match names.as_slice() {
        [name] => is_select_name(name),
        [package, name] => {
            ["tokio", "futures", "crossbeam", "crossbeam_channel"].contains(&package.as_str())
                && is_select_name(name)
        }
        _ => false,
    }
}

pub(crate) fn parse_select_parts(item: &Macro) -> Vec<SelectPart> {
    if !is_select_path(&item.path) {
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
            SelectPart::Pattern(_) | SelectPart::BranchEnd => None,
        })
        .collect()
}
