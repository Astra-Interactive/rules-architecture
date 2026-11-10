use crate::source::model::CfgPredicate;
use std::collections::BTreeMap;
use syn::Meta;
use syn::Token;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

impl CfgPredicate {
    pub(crate) fn in_scope(&self, scope: usize) -> Self {
        match self {
            Self::Atom { key, value } => Self::Atom {
                key: if key.starts_with("target_")
                    || matches!(key.as_str(), "unix" | "windows" | "panic")
                    || key.starts_with("crate_cfg:")
                {
                    key.clone()
                } else {
                    format!("crate_cfg:{scope}:{key}")
                },
                value: value.clone(),
            },
            Self::All(children) => {
                Self::All(children.iter().map(|child| child.in_scope(scope)).collect())
            }
            Self::Any(children) => {
                Self::Any(children.iter().map(|child| child.in_scope(scope)).collect())
            }
            Self::Not(inner) => Self::Not(Box::new(inner.in_scope(scope))),
        }
    }

    pub(crate) fn implies(&self, other: &Self) -> bool {
        !self
            .and(&Self::Not(Box::new(other.clone())))
            .is_satisfiable()
    }
    pub(crate) fn from_attributes(attributes: &[syn::Attribute]) -> Self {
        Self::All(
            attributes
                .iter()
                .filter_map(|attribute| {
                    attribute
                        .path()
                        .is_ident("cfg")
                        .then(|| attribute.parse_args::<Meta>().ok().map(Self::from_meta))
                        .flatten()
                })
                .collect(),
        )
    }

    pub(crate) fn is_test_only(&self) -> bool {
        let test = Self::Atom {
            key: "test".to_owned(),
            value: None,
        };
        self.implies(&test) && test.implies(self)
    }

    fn from_meta(meta: Meta) -> Self {
        match meta {
            Meta::Path(path) => Self::Atom {
                key: crate::source::util::format_tokens(&path),
                value: None,
            },
            Meta::NameValue(pair) => {
                let value = match pair.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(text),
                        ..
                    }) => text.value(),
                    other => crate::source::util::format_tokens(&other),
                };
                Self::Atom {
                    key: crate::source::util::format_tokens(&pair.path),
                    value: Some(value),
                }
            }
            Meta::List(list) => {
                let children =
                    Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone());
                let Ok(children) = children else {
                    return Self::Atom {
                        key: crate::source::util::format_tokens(&list),
                        value: None,
                    };
                };
                let mut children: Vec<Self> = children.into_iter().map(Self::from_meta).collect();
                if list.path.is_ident("any") {
                    return Self::Any(children);
                }
                if list.path.is_ident("not") && children.len() == 1 {
                    return Self::Not(Box::new(children.remove(0)));
                }
                Self::All(children)
            }
        }
    }

    pub(crate) fn find_condition(attribute: &syn::Attribute) -> Option<Self> {
        if attribute.path().is_ident("cfg") {
            return attribute.parse_args::<Meta>().ok().map(Self::from_meta);
        }
        if !attribute.path().is_ident("cfg_attr") {
            return None;
        }
        attribute
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ok()?
            .into_iter()
            .next()
            .map(Self::from_meta)
    }

    pub(crate) fn and(&self, other: &Self) -> Self {
        let mut predicates = Vec::new();
        for predicate in [self, other] {
            match predicate {
                Self::All(children) => predicates.extend(children.iter().cloned()),
                other => predicates.push(other.clone()),
            }
        }
        predicates.dedup();
        Self::All(predicates)
    }

    fn build_alternatives(&self, negated: bool) -> Vec<BTreeMap<String, bool>> {
        match self {
            Self::Not(inner) => inner.build_alternatives(!negated),
            Self::Atom { key, value } => {
                let name = value
                    .as_ref()
                    .map_or_else(|| key.clone(), |value| format!("{key}={value}"));
                vec![BTreeMap::from([(name, !negated)])]
            }
            Self::All(children) | Self::Any(children) => {
                let is_conjunction = matches!(self, Self::All(_)) != negated;
                if !is_conjunction {
                    return children
                        .iter()
                        .flat_map(|child| child.build_alternatives(negated))
                        .collect();
                }
                let mut alternatives = vec![BTreeMap::new()];
                for child in children {
                    let next = child.build_alternatives(negated);
                    alternatives = alternatives
                        .iter()
                        .flat_map(|left| {
                            next.iter()
                                .filter_map(|right| Self::merge_assignment(left, right))
                        })
                        .collect();
                }
                alternatives
            }
        }
    }

    fn merge_assignment(
        left: &BTreeMap<String, bool>,
        right: &BTreeMap<String, bool>,
    ) -> Option<BTreeMap<String, bool>> {
        let mut merged = left.clone();
        for (name, value) in right {
            if merged.get(name).is_some_and(|previous| previous != value) {
                return None;
            }
            let rival = match name.as_str() {
                "unix" => Some("windows"),
                "windows" => Some("unix"),
                _ => None,
            };
            if *value && rival.is_some_and(|rival| merged.get(rival) == Some(&true)) {
                return None;
            }
            let key = name.split_once('=').map(|(key, _)| key);
            if *value
                && matches!(
                    key,
                    Some(
                        "target_arch"
                            | "target_os"
                            | "target_env"
                            | "target_vendor"
                            | "target_endian"
                            | "target_pointer_width"
                            | "target_abi"
                    )
                )
            {
                let key = key?;
                if merged.iter().any(|(existing, enabled)| {
                    *enabled
                        && existing != name
                        && existing
                            .split_once('=')
                            .is_some_and(|(other, _)| other == key)
                }) {
                    return None;
                }
            }
            merged.insert(name.clone(), *value);
        }
        Some(merged)
    }

    pub(crate) fn is_satisfiable(&self) -> bool {
        !self.build_alternatives(false).is_empty()
    }
}
