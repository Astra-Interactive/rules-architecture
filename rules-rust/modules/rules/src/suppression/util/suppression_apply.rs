use crate::catalogue::util::find_catalogue_entry;
use crate::check::model::AcceptedException;
use crate::check::model::Anchor;
use crate::check::model::Location;
use crate::check::model::RuleFinding;
use crate::config::model::ManifestScope;
use crate::config::model::ProjectConfig;
use crate::source::model::ExpectationPosition;
use crate::source::model::ModuleId;
use crate::source::model::NodeSpan;
use crate::suppression::model::ExpectationSlot;
use crate::suppression::model::RuleActivity;
use crate::suppression::model::SuppressionOutcome;
use crate::suppression::model::Suppressor;
use crate::suppression::model::ValidatedExpectations;
use crate::suppression::util::build_meta_finding;
use std::collections::BTreeSet;

fn find_node_scope(slot: &ExpectationSlot, span: &NodeSpan) -> Option<NodeSpan> {
    match &slot.position {
        ExpectationPosition::Node(scope) if scope.contains(span) => Some(*scope),
        ExpectationPosition::ModuleDeclaration { declaration, .. } if declaration == span => {
            Some(*declaration)
        }
        _ => None,
    }
}

fn find_node_slot(
    slots: &[ExpectationSlot],
    rule_id: &str,
    file: &str,
    span: &NodeSpan,
) -> Option<usize> {
    slots
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.rule_id == rule_id && slot.file == file)
        .filter_map(|(index, slot)| find_node_scope(slot, span).map(|scope| (index, scope)))
        .reduce(|(best, outer), (candidate, inner)| {
            if outer.contains(&inner) {
                (candidate, inner)
            } else {
                (best, outer)
            }
        })
        .map(|(index, _)| index)
}

fn find_module_slot(
    slots: &[ExpectationSlot],
    rule_id: &str,
    module: &ModuleId,
    file: &str,
    span: &NodeSpan,
    body_file: &str,
) -> Option<usize> {
    let inner = slots.iter().position(|slot| {
        let is_inner = match &slot.position {
            ExpectationPosition::ModuleInner(inner) => inner == module && slot.file == body_file,
            ExpectationPosition::InlineModuleInner {
                module: inner,
                declaration,
            } => inner == module && slot.file == file && declaration == span,
            _ => false,
        };
        slot.rule_id == rule_id && is_inner
    });
    inner.or_else(|| {
        slots.iter().position(|slot| {
            let is_declaration = matches!(
                &slot.position,
                ExpectationPosition::ModuleDeclaration { module: declared, declaration } if declared == module && declaration == span
            );
            slot.rule_id == rule_id && slot.file == file && is_declaration
        })
    })
}

fn find_suppressor(
    finding: &RuleFinding,
    slots: &[ExpectationSlot],
    config: &ProjectConfig,
) -> Option<Suppressor> {
    let rule_id = finding.finding.rule_id.0.as_str();
    let subject = finding.finding.subject.as_str();
    let manifest = |scope: ManifestScope| {
        config
            .manifest_exceptions
            .iter()
            .position(|exception| {
                let has_rule = exception.rule_id == rule_id;
                let has_scope = exception.scope == scope;
                let has_subject = exception.subject == subject;
                has_rule && has_scope && has_subject
            })
            .map(Suppressor::Manifest)
    };
    match &finding.anchor {
        Anchor::Node { file, span } => {
            find_node_slot(slots, rule_id, file, span).map(Suppressor::Attribute)
        }
        Anchor::Module {
            module,
            file,
            body_file,
            declaration,
        } => find_module_slot(slots, rule_id, module, file, declaration, body_file)
            .map(Suppressor::Attribute),
        Anchor::File(file) => slots
            .iter()
            .position(|slot| {
                let has_rule = slot.rule_id == rule_id;
                let has_file = slot.file == *file;
                let has_file_scope = matches!(slot.position, ExpectationPosition::File);
                has_rule && has_file && has_file_scope
            })
            .map(Suppressor::Attribute),
        Anchor::Crate(name) => manifest(ManifestScope::Package(name.clone())),
        Anchor::Edge {
            from_crate,
            to_crate,
        } => manifest(ManifestScope::Package(from_crate.clone())).or_else(|| {
            config
                .allowed_dependencies
                .iter()
                .position(|dependency| {
                    dependency.from_crate == *from_crate && dependency.to_crate == *to_crate
                })
                .map(Suppressor::Dependency)
        }),
        Anchor::Workspace => manifest(ManifestScope::Workspace),
        Anchor::Unexpectable => None,
    }
}

fn build_accepted(
    finding: &RuleFinding,
    suppressor: Suppressor,
    slots: &[ExpectationSlot],
    config: &ProjectConfig,
) -> Option<AcceptedException> {
    let (path, line, reason) = match suppressor {
        Suppressor::Attribute(index) => {
            let slot = slots.get(index)?;
            (slot.file.clone(), slot.line, slot.reason.clone())
        }
        Suppressor::Manifest(index) => {
            let exception = config.manifest_exceptions.get(index)?;
            (
                exception.path.clone(),
                exception.line,
                exception.reason.clone(),
            )
        }
        Suppressor::Dependency(index) => {
            let dependency = config.allowed_dependencies.get(index)?;
            (
                dependency.path.clone(),
                dependency.line,
                dependency.reason.clone(),
            )
        }
    };
    Some(AcceptedException {
        rule_id: finding.finding.rule_id.clone(),
        location: Location {
            path,
            line: Some(line),
        },
        subject: finding.finding.subject.clone(),
        reason,
    })
}

fn describe_unfulfilled(rule_id: &str, activity: &RuleActivity) -> String {
    let reads_platform_crates =
        find_catalogue_entry(rule_id).is_some_and(|entry| entry.reads_platform_crates);
    if reads_platform_crates && activity.is_platform_words_empty {
        if rule_id == "PlatformImport" {
            return "`PlatformImport` has no configured platform words: its platform checks are inactive, and no technology-path finding is anchored in this scope".to_owned();
        }
        return format!(
            "`{rule_id}` reads platform crates and `platform-words` is empty, so it reports nothing"
        );
    }
    if !activity.active_rules.contains(rule_id) {
        let phase = find_catalogue_entry(rule_id).map_or(0, |entry| entry.phase);
        return format!("`{rule_id}` is not active in klibs-rules: it ships in phase {phase}");
    }
    format!("no `{rule_id}` finding is anchored in the scope of this exception")
}

fn report_unfulfilled(
    slots: &[ExpectationSlot],
    config: &ProjectConfig,
    used: &BTreeSet<Suppressor>,
    activity: &RuleActivity,
    outcome: &mut SuppressionOutcome,
) {
    for (index, slot) in slots.iter().enumerate() {
        if used.contains(&Suppressor::Attribute(index)) {
            continue;
        }
        let message = describe_unfulfilled(&slot.rule_id, activity);
        let finding = build_meta_finding(
            "UnfulfilledExpectation",
            &slot.file,
            slot.line,
            slot.rule_id.clone(),
            message,
        );
        outcome.violations.push(finding);
    }
    for (index, exception) in config.manifest_exceptions.iter().enumerate() {
        if used.contains(&Suppressor::Manifest(index)) {
            continue;
        }
        let message = describe_unfulfilled(&exception.rule_id, activity);
        let subject = format!("{} \"{}\"", exception.rule_id, exception.subject);
        let finding = build_meta_finding(
            "UnfulfilledExpectation",
            &exception.path,
            exception.line,
            subject,
            message,
        );
        outcome.violations.push(finding);
    }
    for (index, dependency) in config.allowed_dependencies.iter().enumerate() {
        if used.contains(&Suppressor::Dependency(index)) {
            continue;
        }
        let message = describe_unfulfilled("UnitDependency", activity);
        let subject = format!("{} -> {}", dependency.from_crate, dependency.to_crate);
        let finding = build_meta_finding(
            "UnfulfilledExpectation",
            &dependency.path,
            dependency.line,
            subject,
            message,
        );
        outcome.violations.push(finding);
    }
}

pub(crate) fn apply_suppressions(
    findings: Vec<RuleFinding>,
    validated: ValidatedExpectations,
    config: &ProjectConfig,
    activity: &RuleActivity,
) -> SuppressionOutcome {
    let mut outcome = SuppressionOutcome {
        violations: validated.findings,
        accepted: Vec::new(),
    };
    let slots = validated.slots;
    let mut used: BTreeSet<Suppressor> = BTreeSet::new();
    for finding in findings {
        let accepted = find_suppressor(&finding, &slots, config).and_then(|suppressor| {
            used.insert(suppressor);
            build_accepted(&finding, suppressor, &slots, config)
        });
        match accepted {
            Some(accepted) => outcome.accepted.push(accepted),
            None => outcome.violations.push(finding.finding),
        }
    }
    report_unfulfilled(&slots, config, &used, activity, &mut outcome);
    outcome
}
