use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::model::HiddenMember;
use crate::module::rule::{DocHiddenVisitor, VisibilityTier};
use crate::module::util::{find_shape_visibility, owns_implementation};
use crate::source::model::CrateIndex;
use crate::source::model::IndexedItem;
use crate::source::util::find_node_span;
use std::collections::BTreeSet;
use syn::spanned::Spanned;

pub(crate) struct DocHiddenUse;

impl DocHiddenUse {
    fn hidden_attributes(item: &syn::Item) -> &[syn::Attribute] {
        let (visibility, attributes) = match item {
            syn::Item::Struct(item) => (&item.vis, &item.attrs),
            syn::Item::Enum(item) => (&item.vis, &item.attrs),
            syn::Item::Union(item) => (&item.vis, &item.attrs),
            syn::Item::Trait(item) => (&item.vis, &item.attrs),
            syn::Item::Type(item) => (&item.vis, &item.attrs),
            syn::Item::Fn(item) => (&item.vis, &item.attrs),
            syn::Item::Const(item) => (&item.vis, &item.attrs),
            syn::Item::Static(item) => (&item.vis, &item.attrs),
            _ => return &[],
        };
        if matches!(visibility, syn::Visibility::Public(_)) {
            attributes
        } else {
            &[]
        }
    }

    fn can_check_users(context: &RuleContext, ordinal: usize) -> bool {
        let crate_name = &context.indexes[ordinal].crate_name;
        let is_published_library = context
            .workspace
            .members
            .iter()
            .find(|member| &member.name == crate_name)
            .is_some_and(|member| {
                let is_published = member.is_published(&context.workspace);
                !is_published || member.has_rust_library()
            });
        is_published_library && context.can_prove_name_users(ordinal)
    }

    fn check_items(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        index
            .items
            .iter()
            .enumerate()
            .filter(|(identity, item)| {
                Self::hidden_attributes(&item.syntax)
                    .iter()
                    .any(DocHiddenVisitor::hides)
                    && !context.has_external_name_user(ordinal, *identity)
            })
            .map(|(_, item)| {
                build_node_finding(
                    "DocHiddenUse",
                    &item.file,
                    item.span,
                    item.name.clone(),
                    Self::MESSAGE.to_owned(),
                )
            })
            .collect()
    }

    fn describe_impl_member<'a>(
        child: &'a syn::ImplItem,
        file: &'a str,
    ) -> Option<HiddenMember<'a>> {
        let (ident, visibility, attributes, span) = match child {
            syn::ImplItem::Fn(member) => {
                (&member.sig.ident, &member.vis, &member.attrs, member.span())
            }
            syn::ImplItem::Const(member) => {
                (&member.ident, &member.vis, &member.attrs, member.span())
            }
            syn::ImplItem::Type(member) => {
                (&member.ident, &member.vis, &member.attrs, member.span())
            }
            _ => return None,
        };
        Some(HiddenMember {
            file,
            name: ident.to_string(),
            span: find_node_span(span),
            attributes,
            is_public: matches!(visibility, syn::Visibility::Public(_)),
            is_trait_member: false,
        })
    }

    fn describe_trait_member<'a>(
        child: &'a syn::TraitItem,
        file: &'a str,
    ) -> Option<HiddenMember<'a>> {
        let (ident, attributes, span) = match child {
            syn::TraitItem::Fn(member) => (&member.sig.ident, &member.attrs, member.span()),
            syn::TraitItem::Const(member) => (&member.ident, &member.attrs, member.span()),
            syn::TraitItem::Type(member) => (&member.ident, &member.attrs, member.span()),
            _ => return None,
        };
        Some(HiddenMember {
            file,
            name: ident.to_string(),
            span: find_node_span(span),
            attributes,
            is_public: true,
            is_trait_member: true,
        })
    }

    fn collect_members<'a>(
        index: &'a CrateIndex,
        identity: usize,
        item: &'a IndexedItem,
    ) -> Vec<HiddenMember<'a>> {
        let mut members: Vec<HiddenMember> = index
            .implementations
            .iter()
            .filter(|implementation| {
                implementation.syntax.trait_.is_none()
                    && owns_implementation(index, identity, implementation)
            })
            .flat_map(|implementation| {
                implementation.syntax.items.iter().filter_map(move |child| {
                    Self::describe_impl_member(child, &implementation.file)
                })
            })
            .collect();
        if let syn::Item::Trait(contract) = &item.syntax {
            members.extend(
                contract
                    .items
                    .iter()
                    .filter_map(|child| Self::describe_trait_member(child, &item.file)),
            );
        }
        members
    }

    fn collect_implemented_names(context: &RuleContext, crate_name: &str) -> BTreeSet<String> {
        context
            .indexes
            .iter()
            .filter(|index| index.crate_name != crate_name)
            .flat_map(|index| &index.implementations)
            .filter(|implementation| implementation.syntax.trait_.is_some())
            .flat_map(|implementation| &implementation.syntax.items)
            .filter_map(|child| match child {
                syn::ImplItem::Fn(member) => Some(member.sig.ident.to_string()),
                syn::ImplItem::Const(member) => Some(member.ident.to_string()),
                syn::ImplItem::Type(member) => Some(member.ident.to_string()),
                _ => None,
            })
            .collect()
    }

    fn check_members(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let candidates: Vec<HiddenMember> = index
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                matches!(
                    find_shape_visibility(&item.syntax),
                    Some(syn::Visibility::Public(_))
                )
            })
            .flat_map(|(identity, item)| Self::collect_members(index, identity, item))
            .filter(|member| {
                member.is_public && member.attributes.iter().any(DocHiddenVisitor::hides)
            })
            .collect();
        if candidates.is_empty() {
            return Vec::new();
        }
        let foreign_names =
            VisibilityTier::collect_foreign_member_names(context, &index.crate_name);
        let implemented_names = Self::collect_implemented_names(context, &index.crate_name);
        candidates
            .into_iter()
            .filter(|member| {
                let is_named = foreign_names.contains(&member.name);
                let is_implemented =
                    member.is_trait_member && implemented_names.contains(&member.name);
                !(is_named || is_implemented)
            })
            .map(|member| {
                build_node_finding(
                    "DocHiddenUse",
                    member.file,
                    member.span,
                    member.name,
                    Self::MESSAGE.to_owned(),
                )
            })
            .collect()
    }

    const MESSAGE: &'static str =
        "reserve doc(hidden) public visibility for actual consumers in another crate";
}

impl Rule for DocHiddenUse {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            let published_library = context
                .workspace
                .members
                .iter()
                .find(|member| member.name == sources.crate_name)
                .is_some_and(|member| {
                    member.is_published(&context.workspace) && member.has_rust_library()
                });
            if !published_library {
                continue;
            }
            for file in sources.files.iter().filter(|file| !file.is_generated) {
                let Some(index) = context.indexes.iter().find(|index| {
                    index.crate_name == file.module.crate_name && index.target == file.module.target
                }) else {
                    continue;
                };
                let Some(module) = index
                    .modules
                    .iter()
                    .find(|module| module.module == file.module)
                else {
                    continue;
                };
                findings.extend(DocHiddenVisitor::collect(file, module.cfg.clone()));
            }
        }
        for ordinal in 0..context.indexes.len() {
            if Self::can_check_users(context, ordinal) {
                findings.extend(Self::check_items(context, ordinal));
                findings.extend(Self::check_members(context, ordinal));
            }
        }
        findings
    }
}
