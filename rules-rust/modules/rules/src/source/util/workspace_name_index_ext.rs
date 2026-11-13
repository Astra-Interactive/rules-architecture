use crate::check::model::RuleContext;
use crate::source::model::{
    CfgPredicate, CrateIndex, NodeSpan, ReferenceSite, Resolution, SourceReference, SourceTree,
    WorkspaceNameIndex, WorkspaceResolvedPath, WorkspaceUse, WorkspaceUseKind,
};
use crate::source::syn::ReferenceSiteVisitor;
use std::collections::BTreeMap;
use syn::visit::Visit;

impl RuleContext {
    pub(crate) fn name_index(&self) -> &WorkspaceNameIndex {
        self.names.get_or_init(|| WorkspaceNameIndex::build(self))
    }
}

fn collect_sites(index: &CrateIndex) -> BTreeMap<(usize, NodeSpan), ReferenceSite> {
    let mut sites = BTreeMap::new();
    for item in &index.items {
        let mut visitor = ReferenceSiteVisitor::new();
        visitor.visit_item(&item.syntax);
        for site in visitor.sites {
            sites.entry((item.module, site.span)).or_insert(site);
        }
    }
    for implementation in &index.implementations {
        let mut visitor = ReferenceSiteVisitor::new();
        visitor.visit_item_impl(&implementation.syntax);
        for site in visitor.sites {
            sites
                .entry((implementation.module, site.span))
                .or_insert(site);
        }
    }
    sites
}

fn expand_target(
    context: &RuleContext,
    target: WorkspaceResolvedPath,
    source_index: usize,
    module: usize,
    site: Option<&ReferenceSite>,
) -> Vec<WorkspaceResolvedPath> {
    let Some(item) = target.path.item else {
        return vec![target];
    };
    let syn::Item::Type(alias) = &context.indexes[target.index].items[item].syntax else {
        return vec![target];
    };
    let resolved = if let Some(site) = site {
        let mut path = site.syntax.clone();
        if site.kind == WorkspaceUseKind::Call && path.segments.len() > 1 {
            path.segments.pop();
        }
        let typed = syn::Type::Path(syn::TypePath {
            attrs: Vec::new(),
            qself: None,
            path,
        });
        context.resolve_outer_workspace_type(source_index, module, &typed)
    } else {
        context.resolve_outer_workspace_type(target.index, target.path.module, &alias.ty)
    };
    let targets = resolved
        .into_iter()
        .map(|mut resolved| {
            resolved.path.cfg = resolved
                .path
                .cfg
                .in_scope(resolved.index)
                .and(&target.path.cfg.in_scope(target.index));
            resolved
        })
        .collect::<Vec<_>>();
    if targets.is_empty() {
        vec![target]
    } else {
        targets
    }
}

fn resolve_reference(
    context: &RuleContext,
    index: usize,
    reference: &SourceReference,
) -> Vec<WorkspaceResolvedPath> {
    match &reference.resolution {
        Resolution::Internal(paths) => paths
            .iter()
            .map(|path| WorkspaceResolvedPath {
                index,
                path: path.clone(),
            })
            .collect(),
        Resolution::External(path) => syn::parse_str::<syn::Path>(path)
            .map(|path| context.resolve_workspace_path(index, reference.module, &path))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn build_reference_uses(
    context: &RuleContext,
    source_index: usize,
    reference: &SourceReference,
    site: Option<&ReferenceSite>,
) -> Vec<WorkspaceUse> {
    let index = &context.indexes[source_index];
    let named_targets = resolve_reference(context, source_index, reference);
    let targets = named_targets
        .iter()
        .cloned()
        .flat_map(|target| expand_target(context, target, source_index, reference.module, site))
        .collect::<Vec<_>>();
    let kind =
        if index.imports.iter().any(|import| {
            import.module == reference.module && import.span.contains(&reference.span)
        }) {
            WorkspaceUseKind::Import
        } else {
            site.map(|site| site.kind)
                .unwrap_or(WorkspaceUseKind::Value)
        };
    targets
        .iter()
        .filter_map(|target| {
            let target_item = target.path.item?;
            let cfg = reference
                .cfg
                .in_scope(source_index)
                .and(&target.path.cfg.in_scope(target.index));
            if !cfg.is_satisfiable() {
                return None;
            }
            let ambiguous = targets.iter().any(|other| {
                (other.index != target.index || other.path.item != target.path.item)
                    && cfg
                        .and(&other.path.cfg.in_scope(other.index))
                        .is_satisfiable()
            });
            let production = CfgPredicate::Not(Box::new(CfgPredicate::Atom {
                key: "test".to_owned(),
                value: None,
            }))
            .in_scope(source_index);
            Some(WorkspaceUse {
                source_index,
                source_module: reference.module,
                file: reference.file.clone(),
                span: reference.span,
                anchor: reference.anchor,
                cfg: cfg.clone(),
                target_index: target.index,
                target_item,
                named_targets: named_targets.clone(),
                kind,
                function: site.and_then(|site| site.function.clone()),
                block: site.and_then(|site| site.block),
                is_test: index.modules[reference.module].tree != SourceTree::Src
                    || !cfg.and(&production).is_satisfiable(),
                ambiguous,
                path: site.map(|site| site.path.clone()).unwrap_or_default(),
            })
        })
        .collect()
}

fn has_name_blind_spots(context: &RuleContext) -> bool {
    let mut availability = crate::source::syn::NameAvailabilityVisitor {
        has_hidden_names: false,
        has_linkable_target: true,
        external_crates: std::collections::BTreeSet::new(),
    };
    for sources in &context.crates {
        availability.has_linkable_target = sources.has_linkable_target;
        availability
            .external_crates
            .clone_from(&sources.external_crates);
        for file in &sources.files {
            availability.visit_file(&file.syntax);
            availability.has_hidden_names |= file.is_generated;
        }
    }
    availability.has_hidden_names
        || context
            .crates
            .iter()
            .any(|sources| !sources.unanalysable.is_empty())
        || context.indexes.iter().any(|index| {
            !index.unanalysable.is_empty()
                || index.imports.iter().any(|import| {
                    import.is_glob
                        && matches!(
                            index.resolve_segments(import.module, &import.path, 0),
                            Resolution::External(_)
                        )
                })
        })
}

impl WorkspaceNameIndex {
    pub(crate) fn build(context: &RuleContext) -> Self {
        let has_blind_spots = has_name_blind_spots(context);
        let mut uses = Vec::new();
        for (source_index, index) in context.indexes.iter().enumerate() {
            let sites = collect_sites(index);
            uses.extend(index.references.iter().flat_map(|reference| {
                build_reference_uses(
                    context,
                    source_index,
                    reference,
                    sites.get(&(reference.module, reference.span)),
                )
            }));
        }
        let mut by_target = BTreeMap::<_, Vec<_>>::new();
        for (position, usage) in uses.iter().enumerate() {
            by_target
                .entry((usage.target_index, usage.target_item))
                .or_default()
                .push(position);
        }
        for (position, usage) in uses.iter().enumerate() {
            for target in &usage.named_targets {
                if let Some(item) = target.path.item
                    && target
                        .path
                        .cfg
                        .in_scope(target.index)
                        .and(&usage.cfg)
                        .is_satisfiable()
                {
                    by_target
                        .entry((target.index, item))
                        .or_default()
                        .push(position);
                }
            }
        }
        for positions in by_target.values_mut() {
            positions.sort_unstable();
            positions.dedup();
        }
        Self {
            uses,
            by_target,
            has_blind_spots,
        }
    }

    pub(crate) fn users_of(
        &self,
        index: usize,
        item: usize,
    ) -> impl Iterator<Item = &WorkspaceUse> {
        self.by_target
            .get(&(index, item))
            .into_iter()
            .flatten()
            .map(|position| &self.uses[*position])
    }
}
