use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::check::util::build_node_finding;
use crate::source::model::{CrateIndex, IndexedImpl, IndexedItem, Resolution};
use crate::workspace::model::TomlValue;
use syn::{Item, Type, Visibility};

impl RuleContext {
    pub(crate) fn is_shape_generated_file(&self, path: &str) -> bool {
        self.crates
            .iter()
            .flat_map(|sources| &sources.files)
            .any(|file| file.path == path && file.is_generated)
    }
}

pub(crate) fn build_shape_item_finding(
    index: &CrateIndex,
    item: &IndexedItem,
    rule: &str,
    message: &str,
) -> RuleFinding {
    let mut path = index.modules[item.module].module.path.clone();
    if index.modules[item.module].is_file_module {
        path.pop();
    }
    path.push(item.name.clone());
    build_node_finding(
        rule,
        &item.file,
        item.span,
        format!(
            "{}::{}",
            index.crate_name.replace('-', "_"),
            path.join("::")
        ),
        message.to_owned(),
    )
}

pub(crate) fn find_kind_name(context: &RuleContext, ordinal: usize, module: usize) -> &str {
    context
        .find_kind_module(ordinal, module)
        .and_then(|kind| context.indexes[ordinal].modules[kind].module.path.last())
        .map(String::as_str)
        .unwrap_or("")
}

pub(crate) fn list_shape_words(context: &RuleContext, crate_name: &str, key: &str) -> Vec<String> {
    let workspace =
        context
            .workspace
            .root_manifest
            .root
            .find(&["workspace", "metadata", "klibs-rules", key]);
    let member = context
        .workspace
        .members
        .iter()
        .find(|member| member.name == crate_name)
        .and_then(|member| {
            member
                .manifest
                .root
                .find(&["package", "metadata", "klibs-rules", key])
        });
    [workspace, member]
        .into_iter()
        .flatten()
        .flat_map(|value| value.as_array().into_iter().flatten())
        .filter_map(TomlValue::as_str)
        .map(str::to_owned)
        .collect()
}

pub(crate) fn find_shape_visibility(item: &Item) -> Option<&Visibility> {
    match item {
        Item::Struct(item) => Some(&item.vis),
        Item::Enum(item) => Some(&item.vis),
        Item::Union(item) => Some(&item.vis),
        Item::Trait(item) => Some(&item.vis),
        Item::Type(item) => Some(&item.vis),
        Item::Fn(item) => Some(&item.vis),
        _ => None,
    }
}

pub(crate) fn find_type_path(ty: &Type) -> Option<&syn::Path> {
    match ty {
        Type::Reference(reference) => find_type_path(&reference.elem),
        Type::Paren(parenthesized) => find_type_path(&parenthesized.elem),
        Type::Group(group) => find_type_path(&group.elem),
        Type::Path(path) if path.qself.is_none() => Some(&path.path),
        _ => None,
    }
}

pub(crate) fn resolve_shape_type(index: &CrateIndex, module: usize, ty: &Type) -> Vec<usize> {
    index.resolve_type_items(module, ty)
}

pub(crate) fn implements_crate_trait(
    index: &CrateIndex,
    item: &IndexedItem,
    implementation: &IndexedImpl,
) -> bool {
    if !item.cfg.and(&implementation.cfg).is_satisfiable() {
        return false;
    }
    let self_types =
        resolve_shape_type(index, implementation.module, &implementation.syntax.self_ty);
    let owns_type = self_types
        .iter()
        .any(|ordinal| std::ptr::eq(&index.items[*ordinal], item));
    let Some((trait_path, _)) = &implementation.syntax.trait_ else {
        return false;
    };
    let trait_exists = match index.resolve_path(implementation.module, trait_path) {
        Resolution::Internal(paths) => {
            paths
                .into_iter()
                .filter_map(|path| path.item)
                .any(|ordinal| {
                    let contract = &index.items[ordinal];
                    matches!(contract.syntax, Item::Trait(_))
                        && contract.cfg.and(&implementation.cfg).is_satisfiable()
                })
        }
        _ => false,
    };
    owns_type && trait_exists
}

pub(crate) fn build_shape_module_finding(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    rule: &str,
    message: &str,
) -> Option<RuleFinding> {
    let index = &context.indexes[ordinal];
    let indexed = &index.modules[module];
    let sources = context
        .crates
        .iter()
        .find(|sources| sources.crate_name == index.crate_name && sources.target == index.target)?;
    let declaration = sources
        .modules
        .iter()
        .find(|candidate| {
            candidate.id == indexed.module
                && candidate.declaration.as_ref().is_some_and(|declaration| {
                    declaration.span == indexed.span && declaration.body_file == indexed.file
                })
        })?
        .declaration
        .as_ref()?;
    Some(RuleFinding {
        finding: Finding {
            rule_id: RuleId(rule.to_owned()),
            location: Location {
                path: declaration.file.clone(),
                line: Some(declaration.span.start.line),
            },
            subject: format!(
                "{}::{}",
                index.crate_name.replace('-', "_"),
                indexed.module.path.join("::")
            ),
            message: message.to_owned(),
        },
        anchor: Anchor::Module {
            module: indexed.module.clone(),
            file: declaration.file.clone(),
            body_file: declaration.body_file.clone(),
            declaration: declaration.span,
        },
    })
}
