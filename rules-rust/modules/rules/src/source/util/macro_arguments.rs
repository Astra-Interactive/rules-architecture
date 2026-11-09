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
