use crate::check::di::RuleModule;
use std::collections::BTreeSet;
use std::sync::Arc;

#[test]
fn given_concept_rule_modules_when_composed_then_all_rules_have_unique_registration_ids() {
    let module = RuleModule::new();
    let ids = module.active_rule_ids();
    assert_eq!(ids.len(), 135);
    assert_eq!(ids.iter().copied().collect::<BTreeSet<_>>().len(), 135);
    assert!(
        ids.iter()
            .all(|id| crate::catalogue::util::find_catalogue_entry(id).is_some())
    );
    assert_eq!(module.rules().len(), ids.len());
}

#[test]
fn given_retired_build_automation_rules_when_composed_then_neither_runs_and_both_are_retired_ids() {
    let module = RuleModule::new();
    let ids = module.active_rule_ids();
    for retired in ["XtaskAlias", "ShellAutomation"] {
        assert!(!ids.contains(&retired), "{retired} still runs");
        assert!(
            crate::catalogue::util::find_catalogue_entry(retired).is_none(),
            "{retired} is still in the catalogue"
        );
        assert!(
            crate::catalogue::util::find_retired_rule(retired).is_some(),
            "{retired} is not a retired id, so an exception naming it is no RetiredRule"
        );
    }
}

#[test]
fn given_constructed_rule_modules_when_services_are_requested_twice_then_the_instances_are_shared()
{
    let module = RuleModule::new();
    let first = module.rules();
    let second = module.rules();
    assert!(
        first
            .iter()
            .zip(&second)
            .all(|(first, second)| Arc::ptr_eq(first, second))
    );
}
