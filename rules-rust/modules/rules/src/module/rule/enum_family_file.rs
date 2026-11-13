use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{
    build_shape_item_finding, owns_implementation, reference_types, resolve_type_uses,
};
use crate::source::model::{CrateIndex, IndexedItem, SourceEdge};
use crate::source::util::find_node_span;
use std::collections::BTreeSet;
use syn::spanned::Spanned;

pub(crate) struct EnumFamilyFile;

impl EnumFamilyFile {
    fn is_payload(index: &CrateIndex, owner: &IndexedItem, identity: usize) -> bool {
        let syn::Item::Enum(value) = &owner.syntax else {
            return false;
        };
        value
            .variants
            .iter()
            .flat_map(|variant| &variant.fields)
            .any(|field| {
                let span = find_node_span(field.ty.span());
                index
                    .references
                    .iter()
                    .filter(|reference| {
                        reference.file == owner.file
                            && reference.span.start >= span.start
                            && reference.span.end <= span.end
                    })
                    .any(|reference| reference_types(index, reference).contains(&identity))
            })
    }

    fn references_type(index: &CrateIndex, edge: &SourceEdge, identity: usize) -> bool {
        let Some(target) = edge.item else {
            return false;
        };
        if target == identity {
            return true;
        }
        let item = &index.items[target];
        matches!(&item.syntax, syn::Item::Type(alias) if resolve_type_uses(index, item.module, &alias.ty, 0).contains(&identity))
    }

    fn belongs_to_family(index: &CrateIndex, identity: usize, item: &IndexedItem) -> Option<usize> {
        let behavior = index.implementations.iter().any(|implementation| {
            owns_implementation(index, identity, implementation)
                && implementation
                    .syntax
                    .items
                    .iter()
                    .any(|item| matches!(item, syn::ImplItem::Fn(_)))
        });
        if behavior {
            return None;
        }
        let owners = index
            .items
            .iter()
            .enumerate()
            .filter(|(_, owner)| {
                owner.cfg.and(&item.cfg).is_satisfiable()
                    && Self::is_payload(index, owner, identity)
            })
            .map(|(identity, _)| identity)
            .collect::<BTreeSet<_>>();
        if owners.len() != 1 {
            return None;
        }
        let owner = *owners.first()?;
        let enumeration = &index.items[owner];
        let other_use = index
            .edges
            .iter()
            .filter(|edge| {
                edge.cfg.and(&item.cfg).is_satisfiable()
                    && Self::references_type(index, edge, identity)
            })
            .any(|edge| {
                let import = index
                    .imports
                    .iter()
                    .any(|import| import.file == edge.file && import.span == edge.span);
                let alias = index.items.iter().any(|item| {
                    matches!(item.syntax, syn::Item::Type(_))
                        && item.file == edge.file
                        && item.span.start <= edge.span.start
                        && item.span.end >= edge.span.end
                });
                let payload = edge.file == enumeration.file
                    && edge.span.start >= enumeration.span.start
                    && edge.span.end <= enumeration.span.end;
                !import && !alias && !payload
            });
        (!other_use).then_some(owner)
    }
}

impl Rule for EnumFamilyFile {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().flat_map(|index| index.items.iter().enumerate().filter(|(_, item)| matches!(item.syntax, syn::Item::Struct(_))).filter_map(|(identity, item)| {
            if context.is_shape_generated_file(&item.file) { return None; }
            let owner = Self::belongs_to_family(index, identity, item)?;
            (item.file != index.items[owner].file && !context.is_shape_generated_file(&index.items[owner].file)).then(|| build_shape_item_finding(index, item, "EnumFamilyFile", "A struct used only by one enum's payloads lives in that enum's file (arch-package.rs.md)"))
        })).collect()
    }
}
