use crate::source::model::{Expectation, ExpectationPosition};
use crate::source::util::format_tokens;
use proc_macro2::{TokenStream, TokenTree};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, Lit, Meta, Token};

pub(crate) struct ExpectationParser {
    file: String,
    line: u32,
    position: ExpectationPosition,
    rule_ids: Vec<String>,
    reason: String,
    malformation: Option<String>,
}

impl ExpectationParser {
    pub(crate) fn new(file: String, line: u32, position: ExpectationPosition) -> Self {
        Self {
            file,
            line,
            position,
            rule_ids: Vec::new(),
            reason: String::new(),
            malformation: None,
        }
    }

    fn into_expectation(self) -> Expectation {
        Expectation {
            file: self.file,
            line: self.line,
            position: self.position,
            rule_ids: self.rule_ids,
            reason: self.reason,
            malformation: self.malformation,
        }
    }

    fn tokens_mention_klibs_rules(tokens: TokenStream) -> bool {
        tokens.into_iter().any(|token| match token {
            TokenTree::Ident(ident) => ident == "klibs_rules",
            TokenTree::Group(group) => Self::tokens_mention_klibs_rules(group.stream()),
            _ => false,
        })
    }

    fn mentions_klibs_rules(meta: &Meta) -> bool {
        let Meta::List(list) = meta else {
            return false;
        };
        let predicate = list.tokens.clone().into_iter().take_while(|token| {
            !matches!(token, TokenTree::Punct(punctuation) if punctuation.as_char() == ',')
        }).collect();
        Self::tokens_mention_klibs_rules(predicate)
    }

    fn check_completeness(&mut self, has_reason: bool) {
        if self.malformation.is_some() {
            return;
        }
        if self.rule_ids.is_empty() {
            self.malformation = Some("the expectation names no rule id".to_owned());
        } else if !has_reason {
            self.malformation = Some("the expectation has no reason".to_owned());
        } else if self.reason.trim().is_empty() {
            self.malformation = Some("the reason is blank".to_owned());
        }
    }

    fn parse_reason(value: &Expr) -> Option<String> {
        let Expr::Lit(literal) = value else {
            return None;
        };
        match &literal.lit {
            Lit::Str(reason) => Some(reason.value()),
            _ => None,
        }
    }

    fn apply_argument(&mut self, argument: Meta) {
        match argument {
            Meta::Path(path) => match path.get_ident() {
                Some(ident) => self.rule_ids.push(ident.to_string()),
                None => {
                    self.malformation = Some(format!(
                        "the rule id `{}` is a path, not one identifier",
                        format_tokens(&path).replace(' ', "")
                    ));
                }
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("reason") => {
                match Self::parse_reason(&name_value.value) {
                    Some(reason) => self.reason = reason,
                    None => {
                        self.malformation = Some("the reason is not a string literal".to_owned());
                    }
                }
            }
            other => {
                self.malformation = Some(format!(
                    "`{}` is not a rule id or a reason",
                    format_tokens(&other)
                ));
            }
        }
    }

    fn parse_expect_arguments(&mut self, list: &syn::MetaList) {
        let Ok(arguments) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            self.malformation = Some("the arguments of `expect` do not parse".to_owned());
            return;
        };
        let has_reason = arguments
            .iter()
            .any(|argument| matches!(argument, Meta::NameValue(name_value) if name_value.path.is_ident("reason")));
        for argument in arguments {
            self.apply_argument(argument);
        }
        self.check_completeness(has_reason);
    }

    pub(crate) fn parse(mut self, attribute: &Attribute) -> Option<Expectation> {
        if !attribute.path().is_ident("cfg_attr") || !Self::mentions_klibs_rules(&attribute.meta) {
            return None;
        }
        let Ok(arguments) =
            attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            self.malformation = Some("the `cfg_attr` arguments do not parse".to_owned());
            return Some(self.into_expectation());
        };
        let mut arguments = arguments.into_iter();
        let predicate = arguments.next();
        let attributes: Vec<Meta> = arguments.collect();
        let is_bare_predicate = predicate
            .as_ref()
            .is_some_and(|meta| matches!(meta, Meta::Path(path) if path.is_ident("klibs_rules")));
        match attributes.as_slice() {
            _ if !is_bare_predicate => {
                self.malformation = Some("the predicate must be the bare `klibs_rules`".to_owned())
            }
            [Meta::List(list)] if list.path.is_ident("expect") => self.parse_expect_arguments(list),
            _ => {
                self.malformation =
                    Some("only `cfg_attr(klibs_rules, expect(…))` is recognised".to_owned())
            }
        }
        Some(self.into_expectation())
    }
}
