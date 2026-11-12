use crate::source::model::SelectPart;
use proc_macro2::{Delimiter, TokenTree};
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;

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

fn is_crossbeam_branch(input: ParseStream) -> bool {
    let fork = input.fork();
    let Ok(operation) = fork.parse::<syn::Ident>() else {
        return false;
    };
    let has_operands = matches!(
        fork.parse::<TokenTree>(),
        Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis
    );
    if !has_operands {
        return false;
    }
    if operation == "default" {
        return fork.peek(syn::Token![=>]);
    }
    (operation == "recv" || operation == "send") && fork.peek(syn::Token![->])
}

fn parse_crossbeam_branch(input: ParseStream, parts: &mut Vec<SelectPart>) -> syn::Result<()> {
    input.parse::<syn::Ident>()?;
    let operands;
    syn::parenthesized!(operands in input);
    for operand in Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated(&operands)? {
        parts.push(SelectPart::Expression(operand));
    }
    if input.peek(syn::Token![->]) {
        input.parse::<syn::Token![->]>()?;
        let pattern = syn::Pat::parse_multi_with_leading_vert(input)?;
        parts.push(SelectPart::Pattern(pattern));
    }
    Ok(())
}

fn parse_select_branch(input: ParseStream, parts: &mut Vec<SelectPart>) -> syn::Result<()> {
    if input.peek(syn::Token![else]) {
        input.parse::<syn::Token![else]>()?;
    } else if input.peek(syn::Ident) && input.peek2(syn::Token![=>]) {
        input.parse::<syn::Ident>()?;
    } else if is_crossbeam_branch(input) {
        parse_crossbeam_branch(input, parts)?;
    } else {
        let pattern = syn::Pat::parse_multi_with_leading_vert(input)?;
        input.parse::<syn::Token![=]>()?;
        parts.push(SelectPart::Expression(input.parse()?));
        parts.push(SelectPart::Pattern(pattern));
        if input.peek(syn::Token![,]) && input.peek2(syn::Token![if]) {
            input.parse::<syn::Token![,]>()?;
            input.parse::<syn::Token![if]>()?;
            parts.push(SelectPart::Expression(input.parse()?));
        }
    }
    input.parse::<syn::Token![=>]>()?;
    parts.push(SelectPart::Expression(parse_select_handler(input)?));
    input.parse::<Option<syn::Token![,]>>()?;
    parts.push(SelectPart::BranchEnd);
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
