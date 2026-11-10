use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::model::{CycleComponent, CycleSearchStep};
use crate::source::model::{CfgPredicate, CrateIndex, SourceEdge};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct PackageDependencyCycle;

impl PackageDependencyCycle {
    fn build_adjacency(index: &CrateIndex, kinds: &[Option<usize>]) -> Vec<Vec<usize>> {
        let mut adjacency = vec![Vec::new(); index.modules.len()];
        for (position, edge) in index.edges.iter().enumerate() {
            let same_tree = index.modules[edge.source].tree == index.modules[edge.target].tree;
            if edge.is_test && !same_tree {
                continue;
            }
            let Some(source) = kinds[edge.source] else {
                continue;
            };
            if kinds[edge.target].is_some_and(|target| target != source) {
                adjacency[source].push(position);
            }
        }
        adjacency
    }

    fn find_cycle(
        index: &CrateIndex,
        adjacency: &[Vec<usize>],
        kinds: &[Option<usize>],
        position: usize,
    ) -> Option<Vec<usize>> {
        let first = &index.edges[position];
        let start = kinds[first.source]?;
        let mut pending = vec![CycleSearchStep {
            module: kinds[first.target]?,
            cfg: first.cfg.clone(),
            edges: vec![position],
        }];
        let mut reached: BTreeMap<usize, Vec<CfgPredicate>> = BTreeMap::new();
        while let Some(step) = pending.pop() {
            let previous = reached.entry(step.module).or_default();
            if previous.iter().any(|cfg| step.cfg.implies(cfg)) {
                continue;
            }
            previous.retain(|cfg| !cfg.implies(&step.cfg));
            previous.push(step.cfg.clone());
            for position in adjacency[step.module].iter().rev() {
                let edge = &index.edges[*position];
                let cfg = step.cfg.and(&edge.cfg);
                if !cfg.is_satisfiable() {
                    continue;
                }
                let Some(target) = kinds[edge.target] else {
                    continue;
                };
                let mut edges = step.edges.clone();
                edges.push(*position);
                if target == start {
                    return Some(edges);
                }
                pending.push(CycleSearchStep {
                    module: target,
                    cfg,
                    edges,
                });
            }
        }
        None
    }

    fn describe_edge(index: &CrateIndex, edge: &SourceEdge) -> String {
        format!(
            "{} -> {} at {}:{}",
            index.modules[edge.source].module.path.join("::"),
            index.modules[edge.target].module.path.join("::"),
            edge.file,
            edge.span.start.line
        )
    }

    fn build_cycle_finding(context: &RuleContext, ordinal: usize, cycle: &[usize]) -> RuleFinding {
        let index = &context.indexes[ordinal];
        let first = &index.edges[cycle[0]];
        let members: BTreeSet<usize> = cycle
            .iter()
            .filter_map(|position| context.find_kind_module(ordinal, index.edges[*position].source))
            .collect();
        let subject = members
            .iter()
            .map(|member| index.modules[*member].module.path.join("::"))
            .collect::<Vec<_>>()
            .join(" <-> ");
        let message = cycle
            .iter()
            .map(|position| Self::describe_edge(index, &index.edges[*position]))
            .collect::<Vec<_>>()
            .join("; ");
        build_node_finding(
            "PackageDependencyCycle",
            &first.file,
            first.span,
            subject,
            message,
        )
    }

    fn build_component(
        index: &CrateIndex,
        kinds: &[Option<usize>],
        cycle: Vec<usize>,
    ) -> CycleComponent {
        CycleComponent {
            modules: cycle
                .iter()
                .filter_map(|position| kinds[index.edges[*position].source])
                .collect(),
            cfg: cycle
                .iter()
                .fold(CfgPredicate::All(Vec::new()), |cfg, position| {
                    cfg.and(&index.edges[*position].cfg)
                }),
            edges: cycle.into_iter().collect(),
        }
    }

    fn merge_component(components: &mut Vec<CycleComponent>, mut component: CycleComponent) {
        let mut position = 0;
        while position < components.len() {
            if components[position].modules.is_disjoint(&component.modules)
                || !components[position]
                    .cfg
                    .and(&component.cfg)
                    .is_satisfiable()
            {
                position += 1;
                continue;
            }
            let previous = components.remove(position);
            component.modules.extend(previous.modules);
            component.edges.extend(previous.edges);
            component.cfg = component.cfg.and(&previous.cfg);
            position = 0;
        }
        components.push(component);
    }

    fn build_components(context: &RuleContext, ordinal: usize) -> Vec<CycleComponent> {
        let mut components: Vec<CycleComponent> = Vec::new();
        let index = &context.indexes[ordinal];
        let kinds: Vec<Option<usize>> = index
            .modules
            .iter()
            .map(|module| context.find_kind_module(ordinal, module.id))
            .collect();
        let adjacency = Self::build_adjacency(index, &kinds);
        for position in adjacency.iter().flatten() {
            let edge = &index.edges[*position];
            let Some(source) = kinds[edge.source] else {
                continue;
            };
            let Some(target) = kinds[edge.target] else {
                continue;
            };
            let covered = components.iter().any(|component| {
                if !component.modules.contains(&source) {
                    return false;
                }
                if !component.modules.contains(&target) {
                    return false;
                }
                component.cfg.and(&edge.cfg).is_satisfiable()
            });
            if covered {
                continue;
            }
            let Some(cycle) = Self::find_cycle(index, &adjacency, &kinds, *position) else {
                continue;
            };
            Self::merge_component(&mut components, Self::build_component(index, &kinds, cycle));
        }
        components
    }
}

impl Rule for PackageDependencyCycle {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for ordinal in 0..context.indexes.len() {
            findings.extend(
                Self::build_components(context, ordinal)
                    .iter()
                    .map(|component| {
                        Self::build_cycle_finding(
                            context,
                            ordinal,
                            &component.edges.iter().copied().collect::<Vec<_>>(),
                        )
                    }),
            );
        }
        findings
    }
}
