use crate::source::model::MacroBody;
use crate::source::model::SelectPart;
use crate::source::util::parse_select_arguments;
use proc_macro2::Punct;
use proc_macro2::Spacing;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use syn::Block;
use syn::Expr;
use syn::Item;
use syn::Macro;
use syn::Stmt;
use syn::Token;
use syn::parse::ParseStream;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit::Visit;

pub(crate) fn try_parse_macro_arguments(item: &Macro) -> Option<Vec<Expr>> {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let arguments = parser.parse2(item.tokens.clone()).ok()?;
    Some(arguments.into_iter().collect())
}

pub(crate) fn parse_macro_arguments(item: &Macro) -> Vec<Expr> {
    try_parse_macro_arguments(item).unwrap_or_default()
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

fn parse_repeat_arguments(input: ParseStream) -> syn::Result<Vec<Expr>> {
    let element: Expr = input.parse()?;
    input.parse::<Token![;]>()?;
    let count: Expr = input.parse()?;
    Ok(vec![element, count])
}

fn parse_macro_repeat(item: &Macro) -> Vec<Expr> {
    parse_repeat_arguments
        .parse2(item.tokens.clone())
        .unwrap_or_default()
}

fn find_last_token_span(tokens: &TokenStream) -> Span {
    match tokens.clone().into_iter().last() {
        Some(TokenTree::Group(group)) => group.span_close(),
        Some(token) => token.span(),
        None => Span::call_site(),
    }
}

fn terminate_statement(tokens: &TokenStream) -> TokenStream {
    let mut semicolon = Punct::new(';', Spacing::Alone);
    semicolon.set_span(find_last_token_span(tokens));
    let mut terminated = tokens.clone();
    terminated.extend([TokenTree::Punct(semicolon)]);
    terminated
}

fn parse_macro_items(item: &Macro) -> Vec<Item> {
    let items = syn::parse2::<syn::File>(item.tokens.clone())
        .or_else(|_| syn::parse2::<syn::File>(terminate_statement(&item.tokens)));
    items.map(|file| file.items).unwrap_or_default()
}

fn parse_macro_statements(item: &Macro) -> Vec<Stmt> {
    Block::parse_within
        .parse2(item.tokens.clone())
        .or_else(|_| Block::parse_within.parse2(terminate_statement(&item.tokens)))
        .unwrap_or_default()
}

fn is_let_led(expression: &Expr) -> bool {
    match expression {
        Expr::Let(_) => true,
        Expr::Binary(inner) => is_let_led(&inner.left),
        Expr::Assign(inner) => is_let_led(&inner.left),
        Expr::Range(inner) => inner.start.as_deref().is_some_and(is_let_led),
        Expr::Cast(inner) => is_let_led(&inner.expr),
        Expr::Try(inner) => is_let_led(&inner.expr),
        Expr::Await(inner) => is_let_led(&inner.base),
        Expr::Field(inner) => is_let_led(&inner.base),
        Expr::Index(inner) => is_let_led(&inner.expr),
        Expr::Call(inner) => is_let_led(&inner.func),
        Expr::MethodCall(inner) => is_let_led(&inner.receiver),
        _ => false,
    }
}

fn find_expression_body(expressions: Vec<Expr>) -> Option<MacroBody> {
    let has_let = expressions.iter().any(is_let_led);
    let is_usable = !expressions.is_empty() && !has_let;
    is_usable.then_some(MacroBody::Expressions(expressions))
}

fn parse_macro_body(item: &Macro) -> MacroBody {
    if let Some(body) = find_expression_body(parse_macro_expressions(item)) {
        return body;
    }
    if let Some(body) = find_expression_body(parse_macro_repeat(item)) {
        return body;
    }
    let items = parse_macro_items(item);
    if !items.is_empty() {
        return MacroBody::Items(items);
    }
    let statements = parse_macro_statements(item);
    if !statements.is_empty() {
        return MacroBody::Statements(statements);
    }
    MacroBody::Opaque
}

pub(crate) fn visit_macro_body<V>(visitor: &mut V, item: &Macro)
where
    V: for<'ast> Visit<'ast>,
{
    match parse_macro_body(item) {
        MacroBody::Expressions(expressions) => {
            for expression in &expressions {
                visitor.visit_expr(expression);
            }
        }
        MacroBody::Items(items) => {
            for declaration in &items {
                visitor.visit_item(declaration);
            }
        }
        MacroBody::Statements(statements) => {
            for statement in &statements {
                visitor.visit_stmt(statement);
            }
        }
        MacroBody::Opaque => {}
    }
}
