use crate::source::model::DeclaredVisibility;
use crate::source::model::NodeSpan;
use crate::source::model::TextPosition;
use proc_macro2::Span;
use quote::ToTokens;
use syn::Attribute;
use syn::Expr;
use syn::Lit;
use syn::Meta;
use syn::Visibility;

pub(crate) fn find_node_span(span: Span) -> NodeSpan {
    let start = span.start();
    let end = span.end();
    NodeSpan {
        start: TextPosition {
            line: u32::try_from(start.line).unwrap_or(u32::MAX),
            column: u32::try_from(start.column).unwrap_or(u32::MAX),
        },
        end: TextPosition {
            line: u32::try_from(end.line).unwrap_or(u32::MAX),
            column: u32::try_from(end.column).unwrap_or(u32::MAX),
        },
    }
}

pub(crate) fn format_tokens(tokens: &impl ToTokens) -> String {
    tokens.to_token_stream().to_string()
}

pub(crate) fn find_path_attribute(attributes: &[Attribute]) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        let Meta::NameValue(name_value) = &attribute.meta else {
            return None;
        };
        if !name_value.path.is_ident("path") {
            return None;
        }
        let Expr::Lit(literal) = &name_value.value else {
            return None;
        };
        match &literal.lit {
            Lit::Str(string) => Some(string.value()),
            _ => None,
        }
    })
}

pub(crate) fn find_declared_visibility(visibility: &Visibility) -> DeclaredVisibility {
    match visibility {
        Visibility::Public(_) => DeclaredVisibility::Public,
        Visibility::Restricted(restricted) => {
            let is_crate = restricted.in_token.is_none() && restricted.path.is_ident("crate");
            if is_crate {
                DeclaredVisibility::Crate
            } else {
                DeclaredVisibility::Restricted
            }
        }
        Visibility::Inherited => DeclaredVisibility::Private,
    }
}

pub(crate) fn has_conditional_path(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        let is_cfg_attr = attribute.path().is_ident("cfg_attr");
        let tokens = format_tokens(&attribute.meta).replace(' ', "");
        is_cfg_attr && (tokens.contains(",path=") || tokens.contains("(path="))
    })
}
