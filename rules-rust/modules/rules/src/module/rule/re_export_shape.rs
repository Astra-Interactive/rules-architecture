use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::source::model::{CfgPredicate, CrateIndex, ImportBinding, IndexedItem, Resolution};

pub(crate) struct ReExportShape;

impl ReExportShape {
    fn has_external_binding_user(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        cfg: &CfgPredicate,
    ) -> bool {
        context
            .name_index()
            .users_of(ordinal, identity)
            .any(|usage| {
                usage.source_index != ordinal
                    && !usage.ambiguous
                    && usage.cfg.and(&cfg.in_scope(ordinal)).is_satisfiable()
            })
    }

    fn check_workspace(context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for binding in index.imports.iter().filter(|binding| {
                binding.is_public
                    && binding.cfg.is_satisfiable()
                    && index.modules[binding.module].tree == crate::source::model::SourceTree::Src
            }) {
                let Resolution::Internal(targets) =
                    index.resolve_segments(binding.module, &binding.path, 0)
                else {
                    continue;
                };
                let items = targets
                    .iter()
                    .filter_map(|target| target.item)
                    .collect::<Vec<_>>();
                let unused_public = binding.is_externally_public
                    && context.can_prove_workspace_only(ordinal)
                    && !items.is_empty()
                    && items.iter().all(|identity| {
                        !Self::is_private(&index.items[*identity])
                            && !context.has_external_name_user(ordinal, *identity)
                    });
                let named_restricted = binding.is_crate_public
                    && items.iter().any(|identity| {
                        Self::has_external_binding_user(context, ordinal, *identity, &binding.cfg)
                    });
                if unused_public || named_restricted {
                    findings.push(build_node_finding("ReExportShape",&binding.file,binding.span,binding.name.clone().unwrap_or_else(||"*".to_owned()),"re-export visibility follows whether another workspace crate names its target".to_owned()));
                }
            }
        }
        findings
    }

    fn has_exact_export(item: &IndexedItem, exports: &[&ImportBinding]) -> bool {
        let valid: Vec<_> = exports
            .iter()
            .filter(|export| {
                !export.is_glob && !export.is_renamed && export.name.as_ref() == Some(&item.name)
            })
            .collect();
        let coverage = CfgPredicate::Any(valid.iter().map(|export| export.cfg.clone()).collect());
        if item
            .cfg
            .and(&CfgPredicate::Not(Box::new(coverage)))
            .is_satisfiable()
        {
            return false;
        }
        for (position, export) in exports.iter().enumerate() {
            let cfg = item.cfg.and(&export.cfg);
            if (export.is_glob || export.is_renamed) && cfg.is_satisfiable() {
                return false;
            }
            if exports[position + 1..]
                .iter()
                .any(|other| cfg.and(&other.cfg).is_satisfiable())
            {
                return false;
            }
        }
        true
    }

    fn is_private(item: &IndexedItem) -> bool {
        let visibility = match &item.syntax {
            syn::Item::Struct(item) => &item.vis,
            syn::Item::Enum(item) => &item.vis,
            syn::Item::Union(item) => &item.vis,
            syn::Item::Trait(item) => &item.vis,
            syn::Item::TraitAlias(item) => &item.vis,
            syn::Item::Type(item) => &item.vis,
            syn::Item::Fn(item) => &item.vis,
            syn::Item::Const(item) => &item.vis,
            syn::Item::Static(item) => &item.vis,
            _ => return true,
        };
        matches!(visibility, syn::Visibility::Inherited)
    }

    fn check_index(index: &CrateIndex) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, item) in index.items.iter().enumerate() {
            let module = &index.modules[item.module];
            if !module.is_file_module {
                continue;
            }
            let Some(parent) = module.parent else {
                continue;
            };
            let exports: Vec<_> = index.imports.iter().filter(|binding| {
                if !binding.is_public || binding.module != parent || !item.cfg.and(&binding.cfg).is_satisfiable() { return false; }
                matches!(index.resolve_segments(binding.module, &binding.path, 0), Resolution::Internal(targets) if targets.iter().any(|target| target.item == Some(ordinal)))
            }).collect();
            if Self::is_private(item) {
                for export in exports {
                    findings.push(build_node_finding(
                        "ReExportShape",
                        &export.file,
                        export.span,
                        item.name.clone(),
                        "A private file-module item cannot be re-exported".to_owned(),
                    ));
                }
                continue;
            }
            if !Self::has_exact_export(item, &exports) {
                findings.push(build_node_finding("ReExportShape", &item.file, item.span, item.name.clone(), "A non-private file-module item needs exactly one by-name re-export without `as` or a glob in its kind block".to_owned()));
            }
        }
        findings
    }
}

impl Rule for ReExportShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<_> = context.indexes.iter().flat_map(Self::check_index).collect();
        findings.extend(Self::check_workspace(context));
        findings
    }
}
