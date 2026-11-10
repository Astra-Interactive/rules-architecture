use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::{find_shape_visibility, reference_types};
use crate::source::model::{
    CrateIndex, DeclaredVisibility, IndexedImpl, IndexedItem, NodeSpan, Resolution,
};
use crate::source::util::find_node_span;
use crate::workspace::model::Target;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;

pub(crate) struct VisibilityTier;

impl VisibilityTier {
    fn is_punct(token: Option<&TokenTree>, expected: char) -> bool {
        matches!(token, Some(TokenTree::Punct(punct)) if punct.as_char() == expected)
    }

    fn is_member_position(tokens: &[TokenTree], position: usize, braced: bool) -> bool {
        let previous = position
            .checked_sub(1)
            .and_then(|before| tokens.get(before));
        let next = tokens.get(position + 1);
        let after_dot = Self::is_punct(previous, '.');
        let after_colon = Self::is_punct(previous, ':');
        let before_colon = Self::is_punct(next, ':');
        if after_dot || after_colon || before_colon {
            return true;
        }
        braced
            && (previous.is_none() || Self::is_punct(previous, ','))
            && (next.is_none() || Self::is_punct(next, ','))
    }

    fn collect_inline_names(literal: &str, names: &mut BTreeSet<String>) {
        if !literal.contains('{') {
            return;
        }
        for word in literal
            .split(|character: char| !(character.is_alphanumeric() || character == '_'))
            .filter(|word| !word.is_empty())
        {
            names.insert(word.to_owned());
        }
    }

    fn collect_member_names(stream: TokenStream, braced: bool, names: &mut BTreeSet<String>) {
        let tokens: Vec<TokenTree> = stream.into_iter().collect();
        for (position, token) in tokens.iter().enumerate() {
            match token {
                TokenTree::Group(group) => Self::collect_member_names(
                    group.stream(),
                    group.delimiter() == Delimiter::Brace,
                    names,
                ),
                TokenTree::Ident(ident) if Self::is_member_position(&tokens, position, braced) => {
                    names.insert(ident.to_string());
                }
                TokenTree::Literal(literal) => {
                    Self::collect_inline_names(&literal.to_string(), names);
                }
                _ => {}
            }
        }
    }

    fn collect_file_member_names(index: &CrateIndex) -> FileMemberNames<'_> {
        let mut names = FileMemberNames::new();
        for item in &index.items {
            let file_names = names.entry(&*item.file).or_default();
            Self::collect_member_names(item.syntax.to_token_stream(), false, file_names);
        }
        for implementation in &index.implementations {
            let file_names = names.entry(&*implementation.file).or_default();
            Self::collect_member_names(implementation.syntax.to_token_stream(), false, file_names);
        }
        names
    }

    fn is_used_outside(names: &FileMemberNames, file: &str, name: &str) -> bool {
        names
            .iter()
            .any(|(other, used)| *other != file && used.contains(name))
    }

    fn is_crate_visible(visibility: &syn::Visibility) -> bool {
        matches!(visibility, syn::Visibility::Restricted(restriction) if restriction.path.is_ident("crate"))
    }

    fn declared_fields(item: &IndexedItem) -> Vec<&syn::Field> {
        match &item.syntax {
            syn::Item::Struct(value) => value.fields.iter().collect(),
            syn::Item::Union(value) => value.fields.named.iter().collect(),
            _ => Vec::new(),
        }
    }

    fn build_unshared_method_findings(
        item: &IndexedItem,
        implementation: &IndexedImpl,
        names: &FileMemberNames,
    ) -> Vec<RuleFinding> {
        implementation
            .syntax
            .items
            .iter()
            .filter_map(|child| {
                let syn::ImplItem::Fn(method) = child else {
                    return None;
                };
                (Self::is_crate_visible(&method.vis)
                    && !Self::is_used_outside(names, &item.file, &method.sig.ident.to_string()))
                .then(|| {
                    build_node_finding(
                        "VisibilityTier",
                        &implementation.file,
                        find_node_span(method.span()),
                        item.name.clone(),
                        "A crate-visible method used only in the file of its type is private"
                            .to_owned(),
                    )
                })
            })
            .collect()
    }

    fn crate_visible_members(
        index: &CrateIndex,
        identity: usize,
        item: &IndexedItem,
        names: &FileMemberNames,
    ) -> Vec<RuleFinding> {
        if !index.unanalysable.is_empty() {
            return Vec::new();
        }
        let mut findings: Vec<RuleFinding> = Self::declared_fields(item)
            .into_iter()
            .filter(|field| Self::is_crate_visible(&field.vis))
            .filter(|field| {
                field.ident.as_ref().is_some_and(|ident| {
                    !Self::is_used_outside(names, &item.file, &ident.to_string())
                })
            })
            .map(|field| {
                build_node_finding(
                    "VisibilityTier",
                    &item.file,
                    find_node_span(field.span()),
                    item.name.clone(),
                    "A crate-visible field used only in the file of its type is private".to_owned(),
                )
            })
            .collect();
        findings.extend(
            index
                .implementations
                .iter()
                .filter(|implementation| {
                    crate::module::util::owns_implementation(index, identity, implementation)
                })
                .flat_map(|implementation| {
                    Self::build_unshared_method_findings(item, implementation, names)
                }),
        );
        findings
    }

    fn check_workspace_item(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        item: &IndexedItem,
    ) -> Option<RuleFinding> {
        let visibility = Self::visibility(&item.syntax)?;
        let path_restricted = matches!(visibility, syn::Visibility::Restricted(restriction) if restriction.in_token.is_some());
        if path_restricted {
            return Some(build_node_finding(
                "VisibilityTier",
                &item.file,
                item.span,
                item.name.clone(),
                "visibility is one of pub, pub(crate) or private; `pub(in path)` is not a tier"
                    .to_owned(),
            ));
        }
        let too_public = matches!(visibility, syn::Visibility::Public(_))
            && !context.has_external_name_user(ordinal, identity);
        let crate_only = matches!(visibility,syn::Visibility::Restricted(restriction) if restriction.path.is_ident("crate"));
        let own_file = crate_only
            && context
                .name_index()
                .users_of(ordinal, identity)
                .filter(|usage| usage.cfg.and(&item.cfg.in_scope(ordinal)).is_satisfiable())
                .all(|usage| usage.source_index == ordinal && usage.file == item.file);
        (too_public || own_file).then(|| {
            build_node_finding(
                "VisibilityTier",
                &item.file,
                item.span,
                item.name.clone(),
                "visibility follows the actual workspace users of the item".to_owned(),
            )
        })
    }

    fn check_workspace(context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context
            .indexes
            .iter()
            .enumerate()
            .filter(|(ordinal, _)| context.can_prove_workspace_only(*ordinal))
        {
            if context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role)
                == Some(crate::config::model::Role::EntryPoint)
            {
                continue;
            }
            let names = Self::collect_file_member_names(index);
            for (identity, item) in index.items.iter().enumerate().filter(|(_, item)| {
                item.cfg.is_satisfiable()
                    && index.modules[item.module].tree == crate::source::model::SourceTree::Src
            }) {
                findings.extend(Self::check_workspace_item(context, ordinal, identity, item));
                findings.extend(Self::crate_visible_members(index, identity, item, &names));
            }
        }
        findings
    }

    fn visibility(item: &syn::Item) -> Option<&syn::Visibility> {
        match item {
            syn::Item::Const(value) => Some(&value.vis),
            syn::Item::Static(value) => Some(&value.vis),
            _ => find_shape_visibility(item),
        }
    }

    fn exposed_spans(item: &IndexedItem) -> Vec<NodeSpan> {
        let spans = match &item.syntax {
            syn::Item::Type(value) => vec![value.ty.span()],
            syn::Item::Fn(value) => vec![value.sig.span()],
            syn::Item::Const(value) => vec![value.ty.span()],
            syn::Item::Static(value) => vec![value.ty.span()],
            syn::Item::Struct(value) => value
                .fields
                .iter()
                .filter(|field| matches!(field.vis, syn::Visibility::Public(_)))
                .map(|field| field.ty.span())
                .collect(),
            syn::Item::Union(value) => value
                .fields
                .named
                .iter()
                .filter(|field| matches!(field.vis, syn::Visibility::Public(_)))
                .map(|field| field.ty.span())
                .collect(),
            syn::Item::Enum(value) => value
                .variants
                .iter()
                .flat_map(|variant| &variant.fields)
                .map(|field| field.ty.span())
                .collect(),
            syn::Item::Trait(value) => value
                .items
                .iter()
                .filter_map(|item| match item {
                    syn::TraitItem::Fn(method) => Some(method.sig.span()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        spans.into_iter().map(find_node_span).collect()
    }

    fn exposed_types(index: &CrateIndex, file: &str, spans: &[NodeSpan]) -> BTreeSet<usize> {
        index
            .references
            .iter()
            .filter(|reference| {
                reference.file == file
                    && spans.iter().any(|span| {
                        reference.span.start >= span.start && reference.span.end <= span.end
                    })
            })
            .flat_map(|reference| reference_types(index, reference))
            .collect()
    }

    fn expose_impl_types(
        index: &CrateIndex,
        implementation: &IndexedImpl,
        items: &BTreeSet<usize>,
    ) -> BTreeSet<usize> {
        let receiver = index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .iter()
            .any(|identity| {
                items.contains(identity)
                    && crate::module::util::owns_implementation(index, *identity, implementation)
            });
        if !receiver {
            return BTreeSet::new();
        }
        let exposed_trait = implementation
            .syntax
            .trait_
            .as_ref()
            .is_some_and(
                |(path, _)| match index.resolve_path(implementation.module, path) {
                    Resolution::External(_) => true,
                    Resolution::Internal(paths) => paths
                        .iter()
                        .any(|path| path.item.is_some_and(|identity| items.contains(&identity))),
                    _ => false,
                },
            );
        let spans = implementation
            .syntax
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(method)
                    if exposed_trait || matches!(method.vis, syn::Visibility::Public(_)) =>
                {
                    Some(find_node_span(method.sig.span()))
                }
                syn::ImplItem::Type(value) if exposed_trait => {
                    Some(find_node_span(value.ty.span()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        Self::exposed_types(index, &implementation.file, &spans)
    }

    fn propagate_types(index: &CrateIndex, items: &mut BTreeSet<usize>) {
        let mut exposed = BTreeSet::new();
        for identity in items.iter() {
            let item = &index.items[*identity];
            exposed.extend(Self::exposed_types(
                index,
                &item.file,
                &Self::exposed_spans(item),
            ));
        }
        exposed.extend(
            index
                .implementations
                .iter()
                .flat_map(|implementation| Self::expose_impl_types(index, implementation, items)),
        );
        items.extend(exposed);
    }

    fn public_module(context: &RuleContext, index: &CrateIndex, module: usize) -> bool {
        let indexed = &index.modules[module];
        context
            .crates
            .iter()
            .filter(|sources| {
                sources.crate_name == index.crate_name && sources.target == index.target
            })
            .flat_map(|sources| &sources.modules)
            .any(|source| {
                source.id == indexed.module
                    && source.declaration.as_ref().is_some_and(|declaration| {
                        declaration.span == indexed.span
                            && declaration.body_file == indexed.file
                            && declaration.visibility == DeclaredVisibility::Public
                    })
            })
    }

    fn reachable_types(context: &RuleContext, index: &CrateIndex) -> BTreeSet<usize> {
        if !context.workspace.members.iter().any(|member| {
            member.name == index.crate_name && member.targets.iter().any(Target::is_rust_library)
        }) {
            return BTreeSet::new();
        }
        let mut modules = BTreeSet::from([0]);
        let mut items = BTreeSet::new();
        loop {
            let previous = (modules.len(), items.len());
            for module in &index.modules {
                if module
                    .parent
                    .is_some_and(|parent| modules.contains(&parent))
                    && module.cfg.is_satisfiable()
                    && Self::public_module(context, index, module.id)
                {
                    modules.insert(module.id);
                }
            }
            for (identity, item) in index.items.iter().enumerate() {
                if modules.contains(&item.module)
                    && matches!(
                        Self::visibility(&item.syntax),
                        Some(syn::Visibility::Public(_))
                    )
                    && item.cfg.is_satisfiable()
                {
                    items.insert(identity);
                }
            }
            Self::follow_exports(index, &mut modules, &mut items);
            Self::propagate_types(index, &mut items);
            if previous == (modules.len(), items.len()) {
                break;
            }
        }
        items
    }

    fn follow_exports(
        index: &CrateIndex,
        modules: &mut BTreeSet<usize>,
        items: &mut BTreeSet<usize>,
    ) {
        for import in index
            .imports
            .iter()
            .filter(|import| import.is_externally_public)
        {
            if !modules.contains(&import.module) {
                continue;
            }
            let Resolution::Internal(targets) =
                index.resolve_segments(import.module, &import.path, 0)
            else {
                continue;
            };
            for target in targets
                .iter()
                .filter(|target| import.cfg.and(&target.cfg).is_satisfiable())
            {
                if let Some(identity) = target.item {
                    items.insert(identity);
                } else {
                    modules.insert(target.module);
                }
            }
        }
    }

    fn public_methods(index: &CrateIndex, item: &IndexedItem, identity: usize) -> Vec<RuleFinding> {
        index
            .implementations
            .iter()
            .filter(|implementation| {
                crate::module::util::owns_implementation(index, identity, implementation)
            })
            .flat_map(|implementation| Self::check_methods(item, implementation))
            .collect()
    }

    fn check_methods(item: &IndexedItem, implementation: &IndexedImpl) -> Vec<RuleFinding> {
        implementation
            .syntax
            .items
            .iter()
            .filter_map(|child| {
                let syn::ImplItem::Fn(method) = child else {
                    return None;
                };
                matches!(method.vis, syn::Visibility::Public(_)).then(|| {
                    build_node_finding(
                        "VisibilityTier",
                        &implementation.file,
                        find_node_span(method.span()),
                        item.name.clone(),
                        "A method is not more visible than its type (arch-package.rs.md)"
                            .to_owned(),
                    )
                })
            })
            .collect()
    }

    fn check_index(context: &RuleContext, index: &CrateIndex) -> Vec<RuleFinding> {
        let reachable = Self::reachable_types(context, index);
        let mut findings = Vec::new();
        for (identity, item) in index.items.iter().enumerate() {
            let Some(visibility) = find_shape_visibility(&item.syntax) else {
                continue;
            };
            let public = matches!(visibility, syn::Visibility::Public(_));
            for field in Self::declared_fields(item)
                .into_iter()
                .filter(|field| matches!(field.vis, syn::Visibility::Public(_)))
            {
                if !public
                    || matches!(item.syntax, syn::Item::Struct(_)) && !reachable.contains(&identity)
                {
                    findings.push(build_node_finding("VisibilityTier", &item.file, find_node_span(field.span()), item.name.clone(), "A public field requires a public struct reachable through public modules or re-exports (arch-package.rs.md)".to_owned()));
                }
            }
            if public {
                continue;
            }
            findings.extend(Self::public_methods(index, item, identity));
        }
        findings
    }
}

impl Rule for VisibilityTier {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<_> = context
            .indexes
            .iter()
            .flat_map(|index| Self::check_index(context, index))
            .collect();
        findings.extend(Self::check_workspace(context));
        findings
    }
}

type FileMemberNames<'a> = BTreeMap<&'a str, BTreeSet<String>>;
