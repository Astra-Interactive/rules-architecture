use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::rule::ContractPlacement;
use crate::module::util::build_shape_module_finding;
use crate::module::util::resolve_shape_type;
use crate::source::model::{IndexedModule, SourceTree};

pub(crate) struct KindShape;

impl KindShape {
    fn watch_sender(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        context
            .resolve_external_type_paths_under(ordinal, module, typed, cfg)
            .iter()
            .any(|path| {
                path.package == "tokio"
                    && path.segments.ends_with(&[
                        "sync".to_owned(),
                        "watch".to_owned(),
                        "Sender".to_owned(),
                    ])
            })
    }
    fn is_mixed_unit(context: &RuleContext, ordinal: usize) -> bool {
        matches!(
            context
                .find_role_config(&context.indexes[ordinal].crate_name)
                .and_then(|config| config.role),
            Some(Role::Technology | Role::Platform | Role::CorePlatform)
        )
    }

    fn is_implementation_file(context: &RuleContext, ordinal: usize, module: usize) -> bool {
        if !Self::is_mixed_unit(context, ordinal) {
            return false;
        }
        let index = &context.indexes[ordinal];
        if index.modules[module].parent == Some(0) {
            return false;
        }
        let implemented: std::collections::BTreeSet<usize> = index
            .implementations
            .iter()
            .filter(|implementation| implementation.syntax.trait_.is_some())
            .filter(|implementation| {
                resolve_shape_type(index, implementation.module, &implementation.syntax.self_ty)
                    .iter()
                    .any(|target| index.items[*target].module == implementation.module)
            })
            .map(|implementation| implementation.module)
            .collect();
        if implemented.contains(&module) {
            return true;
        }
        let mut pending = vec![module];
        let mut visited = std::collections::BTreeSet::new();
        let mut used_by_implementation = false;
        while let Some(current) = pending.pop() {
            if !visited.insert(current) {
                continue;
            }
            for edge in index.edges.iter().filter(|edge| {
                edge.target == current
                    && edge.source != current
                    && !edge.is_test
                    && edge.cfg.is_satisfiable()
            }) {
                let reexport = index.imports.iter().any(|binding| {
                    binding.module == edge.source && binding.span == edge.span && binding.is_public
                });
                if reexport {
                    continue;
                }
                if index.modules[edge.source].parent != index.modules[module].parent {
                    return false;
                }
                if implemented.contains(&edge.source) {
                    used_by_implementation = true;
                } else {
                    pending.push(edge.source);
                }
            }
        }
        used_by_implementation
    }

    fn holds_only_state_holders(
        context: &RuleContext,
        ordinal: usize,
        module: &IndexedModule,
    ) -> bool {
        let owned: Vec<_> = context.indexes[ordinal]
            .items
            .iter()
            .filter(|item| context.find_kind_module(ordinal, item.module) == Some(module.id))
            .collect();
        !owned.is_empty()
            && owned
                .iter()
                .all(|item| ContractPlacement::is_app_api_state_holder(context, ordinal, item))
    }

    fn check_role_kind(
        context: &RuleContext,
        ordinal: usize,
        module: &IndexedModule,
    ) -> Option<RuleFinding> {
        if module.tree != SourceTree::Src {
            return None;
        }
        if context.find_kind_module(ordinal, module.id) != Some(module.id) {
            return None;
        }
        let index = &context.indexes[ordinal];
        let name = module.module.path.last()?.as_str();
        let role = context
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role);
        let app_api = !context.is_plugin_product() && role == Some(Role::Api);
        let forbidden_api = app_api
            && !["api", "model", "util", "stub", "di", "usecase"].contains(&name)
            && !Self::holds_only_state_holders(context, ordinal, module);
        let forbidden_helper = Self::is_mixed_unit(context, ordinal)
            && !["table", "entity", "model", "util", "database", "di"].contains(&name)
            && context.is_technology_kind(&index.crate_name, name);
        (forbidden_api || forbidden_helper).then(|| build_shape_module_finding(context, ordinal, module.id, "KindShape", "the crate role does not permit this kind; technology helpers use table/entity/model/util/database/di and app api contains only its public-contract kinds")).flatten()
    }

    fn is_root_file_block(
        index: &crate::source::model::CrateIndex,
        module: &IndexedModule,
    ) -> bool {
        let root_file = &index.modules[0].file;
        let mut current = module;
        while let Some(parent) = current.parent {
            if current.is_file_module || &current.file != root_file {
                return false;
            }
            current = &index.modules[parent];
        }
        true
    }

    fn find_nesting_sites(
        context: &RuleContext,
        ordinal: usize,
        module: &IndexedModule,
        own_kind: bool,
    ) -> Vec<ShapeSite> {
        let mut sites = Vec::new();
        let nested = module
            .parent
            .is_some_and(|parent| context.find_kind_module(ordinal, parent).is_some());
        if own_kind && nested {
            sites.push(ShapeSite {
                module: module.id,
                message: "a kind block is not nested inside another kind",
            });
        }
        let is_concept_level = module.tree == SourceTree::Src && module.module.path.len() == 3;
        if !is_concept_level {
            return sites;
        }
        let is_root_block = Self::is_root_file_block(&context.indexes[ordinal], module);
        if !is_root_block {
            return sites;
        }
        if context.find_kind_module(ordinal, module.id).is_none() {
            sites.push(ShapeSite {
                module: module.id,
                message: "a concept nests at most two levels",
            });
        }
        sites
    }

    fn check_module(
        context: &RuleContext,
        ordinal: usize,
        module: &IndexedModule,
    ) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let own_kind = context.find_kind_module(ordinal, module.id) == Some(module.id);
        let children: Vec<_> = index
            .modules
            .iter()
            .filter(|child| child.parent == Some(module.id))
            .collect();
        let files: Vec<_> = children
            .iter()
            .filter(|child| child.is_file_module)
            .collect();
        let blocks = children.iter().any(|child| !child.is_file_module);
        let mut sites = Vec::new();
        if !own_kind {
            sites.extend(
                files
                    .iter()
                    .filter(|child| !Self::is_implementation_file(context, ordinal, child.id))
                    .map(|child| ShapeSite {
                        module: child.id,
                        message: "a concept holds kind blocks; files live in a named kind",
                    }),
            );
        }
        if !files.is_empty() && blocks && (!Self::is_mixed_unit(context, ordinal) || own_kind) {
            sites.push(ShapeSite {
                module: module.id,
                message: "a block does not mix file modules and inline blocks",
            });
        }
        let has_items = index.items.iter().any(|item| item.module == module.id);
        let has_impls = index
            .implementations
            .iter()
            .any(|implementation| implementation.module == module.id);
        if own_kind && children.is_empty() && !has_items && !has_impls {
            sites.push(ShapeSite {
                module: module.id,
                message: "a kind block is created with its first file, never empty",
            });
        }
        sites.extend(Self::find_nesting_sites(context, ordinal, module, own_kind));
        sites
            .into_iter()
            .filter_map(|ShapeSite { module, message }| {
                build_shape_module_finding(context, ordinal, module, "KindShape", message)
            })
            .collect()
    }
}

impl Rule for KindShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<_> = context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .modules
                    .iter()
                    .filter(|module| !module.is_file_module)
                    .flat_map(move |module| {
                        Self::check_module(context, ordinal, module)
                            .into_iter()
                            .chain(Self::check_role_kind(context, ordinal, module))
                    })
            })
            .collect();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let syn::Item::Struct(structure) = &item.syntax else {
                    continue;
                };
                if !["model", "api", "usecase", "util"].contains(
                    &crate::module::util::find_kind_name(context, ordinal, item.module),
                ) {
                    continue;
                }
                let sender = structure
                    .fields
                    .iter()
                    .filter(|field| {
                        item.cfg
                            .and(&crate::source::model::CfgPredicate::from_attributes(
                                &field.attrs,
                            ))
                            .is_satisfiable()
                    })
                    .any(|field| {
                        Self::watch_sender(
                            context,
                            ordinal,
                            item.module,
                            &field.ty,
                            &item
                                .cfg
                                .and(&crate::source::model::CfgPredicate::from_attributes(
                                    &field.attrs,
                                )),
                        )
                    });
                if sender {
                    findings.push(crate::module::util::build_shape_item_finding(
                        index,
                        item,
                        "KindShape",
                        "a watch sender is owned by its entry-point state holder",
                    ));
                }
            }
        }
        findings
    }
}

struct ShapeSite {
    module: usize,
    message: &'static str,
}
