use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::rule::FactoryReturnVisitor;
use crate::source::model::{CrateIndex, SourceEdge, SourceTree};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct KindImportDirection;

impl KindImportDirection {
    fn is_fake_source(context: &RuleContext, ordinal: usize, edge: &SourceEdge) -> bool {
        let index = &context.indexes[ordinal];
        context
            .find_kind_module(ordinal, edge.source)
            .is_some_and(|source| {
                index.modules[source].module.path.last().map(String::as_str) == Some("fake")
            })
    }

    fn is_unlisted_kind_target(index: &CrateIndex, target: usize) -> bool {
        let mut block = target;
        while index.modules[block].is_file_module {
            let Some(parent) = index.modules[block].parent else {
                return false;
            };
            block = parent;
        }
        let module = &index.modules[block];
        module.tree == SourceTree::Src
            && module.module.path.len() > 1
            && !["api", "model", "util"]
                .contains(&module.module.path.last().map(String::as_str).unwrap_or(""))
    }

    fn check_registration(context: &RuleContext) -> Vec<RuleFinding> {
        context.name_index().uses.iter().filter(|usage| !usage.is_test && !usage.ambiguous && usage.cfg.is_satisfiable()).filter(|usage| ["model","api","usecase"].contains(&crate::module::util::find_kind_name(context,usage.source_index,usage.source_module))).filter(|usage| context.is_platform_registration_trait(usage.target_index,&context.indexes[usage.target_index].items[usage.target_item])).map(|usage|build_node_finding("KindImportDirection",&usage.file,usage.anchor,context.indexes[usage.target_index].items[usage.target_item].name.clone(),"domain models, contracts and use cases do not depend on framework registration interfaces".to_owned())).collect()
    }

    fn is_root_edge_allowed(context: &RuleContext, ordinal: usize, edge: &SourceEdge) -> bool {
        let index = &context.indexes[ordinal];
        if context.has_technology_implementation_role(&index.crate_name) {
            let Some(target) = context.find_kind_module(ordinal, edge.target) else {
                return true;
            };
            let name = index.modules[target]
                .module
                .path
                .last()
                .map(String::as_str)
                .unwrap_or("");
            return ["api", "model", "util", "table", "entity", "database"].contains(&name)
                || context.is_technology_kind(&index.crate_name, name)
                || context.is_role_kind(&index.crate_name, name);
        }
        let is_entry_point = context.project_config.crates.iter().any(|config| {
            config.name == index.crate_name
                && config.role == Some(crate::config::model::Role::EntryPoint)
        });
        if edge.source != 0 || !is_entry_point {
            return true;
        }
        let Some(target) = context.find_kind_module(ordinal, edge.target) else {
            return true;
        };
        let name = index.modules[target]
            .module
            .path
            .last()
            .map(String::as_str)
            .unwrap_or("");
        name == "di" || context.is_entry_kind(&index.crate_name, name)
    }

    fn is_factory_parameter(index: &CrateIndex, edge: &SourceEdge) -> bool {
        index
            .items
            .iter()
            .filter(|item| item.module == edge.source)
            .any(|item| {
                let syn::Item::Trait(contract) = &item.syntax else {
                    return false;
                };
                let Some(product) = item.name.strip_suffix("Factory") else {
                    return false;
                };
                let Some(product) = index.items.iter().position(|candidate| {
                    candidate.module == item.module
                        && candidate.file == item.file
                        && candidate.name == product
                        && matches!(candidate.syntax, syn::Item::Trait(_))
                }) else {
                    return false;
                };
                contract.items.iter().any(|method| {
                    let syn::TraitItem::Fn(method) = method else {
                        return false;
                    };
                    if method.sig.ident != "invoke" {
                        return false;
                    }
                    let syn::ReturnType::Type(_, output) = &method.sig.output else {
                        return false;
                    };
                    let mut visitor = FactoryReturnVisitor::new(index, item.module, product);
                    visitor.visit_type(output);
                    if !visitor.uses_product {
                        return false;
                    }
                    method
                        .sig
                        .inputs
                        .iter()
                        .any(|parameter| find_node_span(parameter.span()).contains(&edge.span))
                })
            })
    }

    fn is_factory_import(index: &CrateIndex, edge: &SourceEdge) -> bool {
        if !index
            .imports
            .iter()
            .any(|binding| binding.module == edge.source && binding.span == edge.span)
        {
            return false;
        }
        let references: Vec<_> = index
            .edges
            .iter()
            .filter(|other| {
                other.source == edge.source
                    && other.target == edge.target
                    && other.item == edge.item
                    && !index
                        .imports
                        .iter()
                        .any(|binding| binding.module == other.source && binding.span == other.span)
            })
            .collect();
        !references.is_empty()
            && references
                .iter()
                .all(|reference| Self::is_factory_parameter(index, reference))
    }

    fn is_trait_edge(index: &CrateIndex, edge: &SourceEdge) -> bool {
        edge.item
            .is_some_and(|item| matches!(index.items[item].syntax, syn::Item::Trait(_)))
    }

    fn is_technology_edge_allowed(
        context: &RuleContext,
        ordinal: usize,
        edge: &SourceEdge,
        source_name: &str,
        target_name: &str,
    ) -> Option<bool> {
        let index = &context.indexes[ordinal];
        let technology_role = context.has_technology_implementation_role(&index.crate_name);
        if source_name == "util"
            && technology_role
            && ["table", "entity", "database"].contains(&target_name)
        {
            return Some(true);
        }
        let source_technology = context.find_technology_module(ordinal, edge.source);
        let target_technology = context.find_technology_module(ordinal, edge.target);
        let implementation_kind = ["table", "entity", "database"].contains(&source_name)
            || context.is_technology_kind(&index.crate_name, source_name);
        if !implementation_kind || (!technology_role && source_technology.is_none()) {
            return None;
        }
        Some(
            ["api", "model", "util"].contains(&target_name)
                || (technology_role && ["table", "entity", "database"].contains(&target_name))
                || context.is_role_kind(&index.crate_name, target_name)
                || (source_technology.is_some() && source_technology == target_technology),
        )
    }

    fn is_allowed(context: &RuleContext, ordinal: usize, edge: &SourceEdge) -> bool {
        let index = &context.indexes[ordinal];
        if edge.source == edge.target {
            return true;
        }
        if edge.is_test && !Self::is_fake_source(context, ordinal, edge) {
            return true;
        }
        let Some(source) = context.find_kind_module(ordinal, edge.source) else {
            return Self::is_root_edge_allowed(context, ordinal, edge);
        };
        let Some(target) = context.find_kind_module(ordinal, edge.target) else {
            if edge.is_test {
                return !Self::is_unlisted_kind_target(index, edge.target);
            }
            let source_name = index.modules[source]
                .module
                .path
                .last()
                .map(String::as_str)
                .unwrap_or("");
            return !context.has_technology_implementation_role(&index.crate_name)
                || !["model", "api", "usecase"].contains(&source_name);
        };
        if source == edge.source
            && index.modules[edge.target].is_file_module
            && index.modules[edge.target].parent == Some(source)
            && index.imports.iter().any(|binding| {
                binding.module == source && binding.is_public && binding.span == edge.span
            })
        {
            return true;
        }
        Self::is_kind_pair_allowed(context, ordinal, edge, source, target)
    }

    fn is_entry_pair_allowed(
        context: &RuleContext,
        crate_name: &str,
        source_name: &str,
        target_name: &str,
    ) -> bool {
        let is_ui_pair = !Self::FIXED_ENTRY_KINDS.contains(&source_name)
            && !Self::FIXED_ENTRY_KINDS.contains(&target_name);
        ["usecase", "api", "model", "util"].contains(&target_name)
            || context.is_role_kind(crate_name, target_name)
            || ((is_ui_pair || source_name == target_name)
                && context.is_entry_kind(crate_name, target_name))
    }

    fn is_kind_pair_allowed(
        context: &RuleContext,
        ordinal: usize,
        edge: &SourceEdge,
        source: usize,
        target: usize,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let source_name = index.modules[source]
            .module
            .path
            .last()
            .map(String::as_str)
            .unwrap_or("");
        let target_name = index.modules[target]
            .module
            .path
            .last()
            .map(String::as_str)
            .unwrap_or("");
        if source_name == "di" {
            return true;
        }
        if target_name == "di" {
            return false;
        }
        if let Some(allowed) =
            Self::is_technology_edge_allowed(context, ordinal, edge, source_name, target_name)
        {
            return allowed;
        }
        let is_role_target = context.is_role_kind(&index.crate_name, target_name);
        match source_name {
            "model" => target_name == "model",
            "api" => {
                target_name == "model"
                    || (target_name == "api"
                        && (Self::is_factory_parameter(index, edge)
                            || Self::is_factory_import(index, edge)))
            }
            "usecase" => ["api", "model", "util"].contains(&target_name) || is_role_target,
            "fake" => ["api", "model", "util"].contains(&target_name),
            "util" => {
                ["model", "util"].contains(&target_name)
                    || (target_name == "api" && !Self::is_trait_edge(index, edge))
            }
            "stub" => {
                ["api", "model", "util"].contains(&target_name)
                    || is_role_target
                    || source == target
            }
            _ if context.is_technology_kind(&index.crate_name, source_name) => {
                ["api", "model", "util"].contains(&target_name)
                    || is_role_target
                    || source == target
            }
            _ if context.is_entry_kind(&index.crate_name, source_name) => {
                Self::is_entry_pair_allowed(context, &index.crate_name, source_name, target_name)
            }
            _ if context.is_role_kind(&index.crate_name, source_name) => {
                ["model", "util"].contains(&target_name) || is_role_target
            }
            _ => true,
        }
    }

    pub(crate) const FIXED_ENTRY_KINDS: [&str; 8] = [
        "command",
        "event",
        "route",
        "job",
        "service",
        "warmup",
        "migration",
        "error",
    ];
}

impl Rule for KindImportDirection {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings:Vec<_>=context.indexes.iter().enumerate().flat_map(|(ordinal, index)| index.edges.iter().filter(move |edge| !Self::is_allowed(context, ordinal, edge)).map(|edge| build_node_finding("KindImportDirection", &edge.file, edge.span, index.modules[edge.source].module.path.join("::"), format!("This kind may not use `{}` (arch-context.md, Direction and enforcement)", index.modules[edge.target].module.path.join("::"))))).collect();
        findings.extend(Self::check_registration(context));
        findings
    }
}
