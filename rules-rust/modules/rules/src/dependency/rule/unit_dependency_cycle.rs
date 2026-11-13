use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::source::model::CfgPredicate;
use crate::workspace::model::{Dependency, DependencyKind, Member, TomlEntry};
use std::collections::{BTreeMap, BTreeSet};
use syn::parse::Parser;

pub(crate) struct UnitDependencyCycle;

impl UnitDependencyCycle {
    fn find_reachable<'graph>(
        start: &'graph str,
        edges: &BTreeMap<&'graph str, BTreeSet<&'graph str>>,
    ) -> BTreeSet<&'graph str> {
        let mut visited = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(name) = pending.pop() {
            if !visited.insert(name) {
                continue;
            }
            if let Some(neighbours) = edges.get(name) {
                pending.extend(neighbours.iter().copied());
            }
        }
        visited
    }

    fn declares_line(target: &TomlEntry, line: u32) -> bool {
        Self::DEPENDENCY_TABLES.into_iter().any(|table| {
            target.value.find(&[table]).is_some_and(|dependencies| {
                dependencies
                    .entries()
                    .iter()
                    .any(|entry| entry.value.line == line)
            })
        })
    }

    fn find_target_key<'member>(
        member: &'member Member,
        dependency: &Dependency,
    ) -> Option<&'member str> {
        if !dependency.is_target_specific {
            return None;
        }
        let line = dependency.line?;
        member
            .manifest
            .root
            .find(&["target"])?
            .entries()
            .iter()
            .find(|target| Self::declares_line(target, line))
            .map(|target| target.key.as_str())
    }

    fn list_triples(members: &[Member]) -> Vec<String> {
        let triples: BTreeSet<String> = members
            .iter()
            .flat_map(|member| {
                member
                    .dependencies
                    .iter()
                    .filter_map(move |dependency| Self::find_target_key(member, dependency))
            })
            .filter(|target| !target.starts_with("cfg("))
            .map(ToOwned::to_owned)
            .collect();
        triples.into_iter().collect()
    }

    fn parse_cfg_condition(target: &str) -> Option<CfgPredicate> {
        let attributes = syn::Attribute::parse_outer
            .parse_str(&format!("#[{target}]"))
            .ok()?;
        CfgPredicate::find_condition(attributes.first()?)
    }

    fn build_triple_atom(triple: &str) -> CfgPredicate {
        CfgPredicate::Atom {
            key: Self::TRIPLE_KEY.to_owned(),
            value: Some(triple.to_owned()),
        }
    }

    fn build_triple_condition(triple: &str, triples: &[String]) -> CfgPredicate {
        let mut parts = vec![Self::build_triple_atom(triple)];
        parts.extend(
            triples
                .iter()
                .filter(|other| other.as_str() != triple)
                .map(|other| CfgPredicate::Not(Box::new(Self::build_triple_atom(other)))),
        );
        CfgPredicate::All(parts)
    }

    fn build_condition(target: Option<&str>, triples: &[String]) -> CfgPredicate {
        let unconditional = CfgPredicate::All(Vec::new());
        let Some(target) = target else {
            return unconditional;
        };
        if target.starts_with("cfg(") {
            return Self::parse_cfg_condition(target).unwrap_or(unconditional);
        }
        Self::build_triple_condition(target, triples)
    }

    fn build_edges(members: &[Member]) -> BTreeMap<&str, Vec<CycleEdge<'_>>> {
        let triples = Self::list_triples(members);
        let mut edges: BTreeMap<&str, Vec<CycleEdge<'_>>> = BTreeMap::new();
        for member in members {
            for dependency in &member.dependencies {
                if dependency.find_member(members).is_none() {
                    continue;
                }
                let target = Self::find_target_key(member, dependency);
                edges.entry(&member.name).or_default().push(CycleEdge {
                    target: &dependency.name,
                    is_development: dependency.kind == DependencyKind::Development,
                    cfg: Self::build_condition(target, &triples),
                });
            }
        }
        edges
    }

    fn closes_cycle<'graph>(
        start: &str,
        first: &CycleEdge<'graph>,
        component: &BTreeSet<&str>,
        edges: &BTreeMap<&'graph str, Vec<CycleEdge<'graph>>>,
    ) -> bool {
        if first.target == start || !first.cfg.is_satisfiable() {
            return false;
        }
        let mut pending = vec![first.clone()];
        let mut reached: BTreeMap<&'graph str, Vec<CfgPredicate>> = BTreeMap::new();
        while let Some(walk) = pending.pop() {
            let previous = reached.entry(walk.target).or_default();
            if previous.iter().any(|cfg| walk.cfg.implies(cfg)) {
                continue;
            }
            previous.retain(|cfg| !cfg.implies(&walk.cfg));
            previous.push(walk.cfg.clone());
            for edge in edges.get(walk.target).into_iter().flatten() {
                let cfg = walk.cfg.and(&edge.cfg);
                if edge.target == walk.target || !component.contains(edge.target) {
                    continue;
                }
                if !cfg.is_satisfiable() {
                    continue;
                }
                if edge.target == start {
                    return true;
                }
                pending.push(CycleEdge {
                    target: edge.target,
                    is_development: edge.is_development,
                    cfg,
                });
            }
        }
        false
    }

    fn has_development_cycle<'graph>(
        component: &BTreeSet<&str>,
        edges: &BTreeMap<&'graph str, Vec<CycleEdge<'graph>>>,
    ) -> bool {
        component.iter().any(|source| {
            edges.get(source).into_iter().flatten().any(|edge| {
                edge.is_development
                    && component.contains(edge.target)
                    && Self::closes_cycle(source, edge, component, edges)
            })
        })
    }

    const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    const TRIPLE_KEY: &str = "target_triple";
}

impl Rule for UnitDependencyCycle {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let members = &context.workspace.members;
        let conditional_edges = Self::build_edges(members);
        let mut edges: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut reverse: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for (source, targets) in &conditional_edges {
            for edge in targets {
                edges.entry(*source).or_default().insert(edge.target);
                reverse.entry(edge.target).or_default().insert(*source);
            }
        }
        let mut visited = BTreeSet::new();
        let mut findings = Vec::new();
        for member in members {
            if visited.contains(member.name.as_str()) {
                continue;
            }
            let forward = Self::find_reachable(&member.name, &edges);
            let backward = Self::find_reachable(&member.name, &reverse);
            let component: BTreeSet<&str> = forward.intersection(&backward).copied().collect();
            visited.extend(component.iter().copied());
            if component.len() < 2 || !Self::has_development_cycle(&component, &conditional_edges) {
                continue;
            }
            findings.push(RuleFinding {
                finding: Finding {
                    rule_id: RuleId("UnitDependencyCycle".to_owned()),
                    location: Location { path: context.workspace.root_manifest.path.clone(), line: None },
                    subject: component.into_iter().collect::<Vec<_>>().join(", "),
                    message: "a development dependency closes a cycle between these crates: each compilation's graph must remain acyclic".to_owned(),
                },
                anchor: Anchor::Unexpectable,
            });
        }
        findings
    }
}

#[derive(Clone)]
struct CycleEdge<'graph> {
    target: &'graph str,
    is_development: bool,
    cfg: CfgPredicate,
}
