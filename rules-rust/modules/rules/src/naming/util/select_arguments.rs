use crate::naming::model::SelectPart;
use syn::parse::ParseStream;

fn parse_select_handler(input: ParseStream) -> syn::Result<syn::Expr> {
    if input.peek(syn::token::Brace) {
        return Ok(syn::Expr::Block(syn::ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: input.parse()?,
        }));
    }
    input.parse()
}

fn parse_select_branch(input: ParseStream, parts: &mut Vec<SelectPart>) -> syn::Result<()> {
    if input.peek(syn::Token![else]) {
        input.parse::<syn::Token![else]>()?;
    } else {
        let pattern = syn::Pat::parse_multi_with_leading_vert(input)?;
        parts.push(SelectPart::Pattern(pattern));
        input.parse::<syn::Token![=]>()?;
        parts.push(SelectPart::Expression(input.parse()?));
        if input.peek(syn::Token![,]) && input.peek2(syn::Token![if]) {
            input.parse::<syn::Token![,]>()?;
            input.parse::<syn::Token![if]>()?;
            parts.push(SelectPart::Expression(input.parse()?));
        }
    }
    input.parse::<syn::Token![=>]>()?;
    parts.push(SelectPart::Expression(parse_select_handler(input)?));
    input.parse::<Option<syn::Token![,]>>()?;
    Ok(())
}

pub(crate) fn parse_select_arguments(input: ParseStream) -> syn::Result<Vec<SelectPart>> {
    let mut parts = Vec::new();
    if input.peek(syn::Ident) && input.peek2(syn::Token![;]) {
        input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![;]>()?;
    }
    while !input.is_empty() {
        parse_select_branch(input, &mut parts)?;
    }
    Ok(parts)
}
