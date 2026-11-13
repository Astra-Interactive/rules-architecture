use crate::check::model::{ApiOwnershipReport, RuleContext};
use crate::config::model::Role;
use crate::module::util::find_kind_name;
use crate::source::model::WorkspaceUseKind;
use crate::workspace::model::Member;
use std::collections::{BTreeMap, BTreeSet};

fn collect_consumers(
    context: &RuleContext,
    index: usize,
    item: usize,
    visited: &mut BTreeMap<(usize, usize), crate::source::model::CfgPredicate>,
    consumers: &mut BTreeSet<String>,
    uncertain: &mut bool,
    cfg: &crate::source::model::CfgPredicate,
) {
    if visited
        .get(&(index, item))
        .is_some_and(|covered| cfg.implies(covered))
    {
        return;
    }
    visited
        .entry((index, item))
        .and_modify(|covered| {
            *covered = crate::source::model::CfgPredicate::Any(vec![covered.clone(), cfg.clone()])
        })
        .or_insert_with(|| cfg.clone());
    for usage in context.name_index().users_of(index, item) {
        if usage.is_test || usage.kind == WorkspaceUseKind::Import {
            continue;
        }
        let predicate = cfg.and(&usage.cfg);
        if !predicate.is_satisfiable() {
            continue;
        }
        *uncertain |= usage.ambiguous;
        let owner = &context.indexes[usage.source_index];
        if find_kind_name(context, usage.source_index, usage.source_module) == "di" {
            *uncertain |= context
                .find_role_config(&owner.crate_name)
                .and_then(|config| config.role)
                != Some(Role::EntryPoint);
            continue;
        }
        if usage.source_index != index {
            consumers.insert(owner.crate_name.clone());
        }
        for (identity, _) in owner.items.iter().enumerate().filter(|(_, declaration)| {
            declaration.file == usage.file && declaration.span.contains(&usage.span)
        }) {
            collect_consumers(
                context,
                usage.source_index,
                identity,
                visited,
                consumers,
                uncertain,
                &predicate,
            );
        }
    }
}

fn classify(
    context: &RuleContext,
    member: &Member,
    consumers: &BTreeSet<String>,
    uncertain: bool,
) -> &'static str {
    if member.is_published(&context.workspace) {
        return "EXTERNAL";
    }
    if uncertain {
        return "UNCERTAIN";
    }
    if consumers.is_empty() {
        return "UNKNOWN";
    }
    if consumers.len() == 1
        && consumers.iter().all(|name| {
            context
                .find_role_config(name)
                .and_then(|config| config.role)
                == Some(Role::Impl)
                && context
                    .workspace
                    .members
                    .iter()
                    .find(|candidate| &candidate.name == name)
                    .is_some_and(|candidate| {
                        candidate.directory.rsplit_once('/').map(|pair| pair.0)
                            == member.directory.rsplit_once('/').map(|pair| pair.0)
                    })
        })
    {
        "CANDIDATE"
    } else {
        "RETAIN"
    }
}

fn render_item(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
    member: &Member,
    blind_spots: bool,
) -> String {
    let index = &context.indexes[ordinal];
    let item = &index.items[identity];
    let mut consumers = BTreeSet::new();
    let mut uncertain = blind_spots;
    collect_consumers(
        context,
        ordinal,
        identity,
        &mut BTreeMap::new(),
        &mut consumers,
        &mut uncertain,
        &item.cfg.in_scope(ordinal),
    );
    let module = crate::check::util::format_module_path(
        &index.crate_name,
        &index.modules[item.module].module.path,
    );
    format!(
        "{}\t{}::{}\t{}:{}\tconsumers={}\tuncertain={}\tcfg={:?}\n",
        classify(context, member, &consumers, uncertain),
        module,
        item.name,
        item.file,
        item.span.start.line,
        consumers.into_iter().collect::<Vec<_>>().join(","),
        uncertain,
        item.cfg
    )
}

pub(crate) fn build_api_ownership(context: &RuleContext) -> ApiOwnershipReport {
    let mut rows = Vec::new();
    let blind_spots = context.name_index().has_blind_spots;
    for (ordinal, index) in context.indexes.iter().enumerate() {
        if !matches!(
            context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role),
            Some(Role::Api | Role::CoreApi | Role::Contract | Role::StorageContract)
        ) {
            continue;
        }
        let Some(member) = context
            .workspace
            .members
            .iter()
            .find(|member| member.name == index.crate_name)
        else {
            continue;
        };
        for (identity, item) in index.items.iter().enumerate() {
            let public = match &item.syntax {
                syn::Item::Struct(item) => &item.vis,
                syn::Item::Enum(item) => &item.vis,
                syn::Item::Trait(item) => &item.vis,
                syn::Item::Type(item) => &item.vis,
                syn::Item::Fn(item) => &item.vis,
                _ => continue,
            };
            if !matches!(public, syn::Visibility::Public(_)) {
                continue;
            }
            rows.push(render_item(context, ordinal, identity, member, blind_spots));
        }
    }
    rows.sort();
    rows.dedup();
    ApiOwnershipReport {
        text: format!(
            "API ownership advisory (syntactic workspace evidence)\nNo automatic moves. Tests and composition-root DI do not establish shared behavior. External consumers and unresolved inference require review.\n{}",
            rows.concat()
        ),
    }
}
