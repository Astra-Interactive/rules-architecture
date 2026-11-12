use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::dependency::util::build_dependency_finding;
use crate::source::model::{CfgPredicate, CrateIndex, Resolution};
use crate::workspace::model::DependencyKind;
use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::ToTokens;

pub(crate) struct PlatformDependencyTable;

impl PlatformDependencyTable {
    fn is_path_separator(trees: &[TokenTree], position: usize) -> bool {
        matches!((trees.get(position), trees.get(position + 1)),
            (Some(TokenTree::Punct(first)), Some(TokenTree::Punct(second)))
                if first.as_char() == ':'
                    && first.spacing() == Spacing::Joint
                    && second.as_char() == ':')
    }

    fn has_macro_bang(trees: &[TokenTree], position: usize) -> bool {
        let mut next = position + 1;
        while Self::is_path_separator(trees, next)
            && matches!(trees.get(next + 2), Some(TokenTree::Ident(_)))
        {
            next += 3;
        }
        next > position + 1
            && matches!(trees.get(next), Some(TokenTree::Punct(bang))
                if bang.as_char() == '!' && bang.spacing() == Spacing::Alone)
    }

    fn is_attribute_group(trees: &[TokenTree], position: usize) -> bool {
        let TokenTree::Group(group) = &trees[position] else {
            return false;
        };
        let punct_before = |distance: usize| match position
            .checked_sub(distance)
            .and_then(|index| trees.get(index))
        {
            Some(TokenTree::Punct(punct)) => Some(punct.as_char()),
            _ => None,
        };
        group.delimiter() == Delimiter::Bracket
            && (punct_before(1) == Some('#')
                || (punct_before(1) == Some('!') && punct_before(2) == Some('#')))
    }

    fn has_path_use(tokens: TokenStream, code_name: &str, in_attribute: bool) -> bool {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        trees.iter().enumerate().any(|(position, tree)| match tree {
            TokenTree::Group(group) => {
                let in_attribute = in_attribute || Self::is_attribute_group(&trees, position);
                Self::has_path_use(group.stream(), code_name, in_attribute)
            }
            TokenTree::Ident(ident) if ident == code_name => {
                let is_use = in_attribute || Self::has_macro_bang(&trees, position);
                is_use && Self::is_path_separator(&trees, position + 1)
            }
            _ => false,
        })
    }

    fn list_macro_predicates(index: &CrateIndex, code_name: &str) -> Vec<CfgPredicate> {
        let items = index
            .items
            .iter()
            .filter(|item| !matches!(item.syntax, syn::Item::Mod(_)))
            .filter(|item| Self::has_path_use(item.syntax.to_token_stream(), code_name, false))
            .map(|item| item.cfg.and(&index.modules[item.module].cfg));
        let implementations = index
            .implementations
            .iter()
            .filter(|implementation| {
                Self::has_path_use(implementation.syntax.to_token_stream(), code_name, false)
            })
            .map(|implementation| {
                implementation
                    .cfg
                    .and(&index.modules[implementation.module].cfg)
            });
        items.chain(implementations).collect()
    }

    fn list_reference_predicates(index: &CrateIndex, code_name: &str) -> Vec<CfgPredicate> {
        let imports = index
            .imports
            .iter()
            .filter(|binding| {
                binding
                    .path
                    .first()
                    .is_some_and(|segment| segment == code_name)
            })
            .map(|binding| binding.cfg.and(&index.modules[binding.module].cfg));
        let references = index
            .references
            .iter()
            .filter(|reference| {
                matches!(&reference.resolution, Resolution::External(path)
                    if path.split("::").next() == Some(code_name))
            })
            .map(|reference| reference.cfg.and(&index.modules[reference.module].cfg));
        imports
            .chain(references)
            .chain(Self::list_macro_predicates(index, code_name))
            .collect()
    }

    fn is_used_only_under_target(index: &CrateIndex, code_name: &str) -> bool {
        let predicates = Self::list_reference_predicates(index, code_name);
        !predicates.is_empty() && predicates.iter().all(CfgPredicate::implies_target)
    }
}

impl Rule for PlatformDependencyTable {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .workspace
            .members
            .iter()
            .flat_map(|member| {
                let index = context
                    .indexes
                    .iter()
                    .find(|index| index.crate_name == member.name);
                member.dependencies.iter().filter_map(move |dependency| {
                    let index = index?;
                    let is_plain = dependency.kind == DependencyKind::Normal
                        && !dependency.is_target_specific;
                    (is_plain && Self::is_used_only_under_target(index, &dependency.code_name))
                        .then(|| {
                            build_dependency_finding(
                                "PlatformDependencyTable",
                                member,
                                dependency,
                                &dependency.name,
                                "a dependency used only under a target cfg sits in [target.'cfg(…)'.dependencies] (arch-package.rs.md)",
                            )
                        })
                })
            })
            .collect()
    }
}
