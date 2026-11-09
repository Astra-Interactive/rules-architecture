use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::{DependencyKind, Member};
use std::collections::{BTreeMap, BTreeSet};

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

    fn has_development_edge(component: &BTreeSet<&str>, members: &[Member]) -> bool {
        members
            .iter()
            .filter(|member| component.contains(member.name.as_str()))
            .any(|member| {
                member.dependencies.iter().any(|dependency| {
                    dependency.kind == DependencyKind::Development
                        && component.contains(dependency.name.as_str())
                        && dependency.find_member(members).is_some()
                })
            })
    }
}

impl Rule for UnitDependencyCycle {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let members = &context.workspace.members;
        let mut edges: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut reverse: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for member in members {
            for dependency in &member.dependencies {
                if dependency.find_member(members).is_none() {
                    continue;
                }
                edges
                    .entry(&member.name)
                    .or_default()
                    .insert(&dependency.name);
                reverse
                    .entry(&dependency.name)
                    .or_default()
                    .insert(&member.name);
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
            if component.len() < 2 || !Self::has_development_edge(&component, members) {
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
