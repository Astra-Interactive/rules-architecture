use crate::check::di::RuleModule;
use std::collections::BTreeSet;
use std::sync::Arc;

#[test]
fn given_concept_rule_modules_when_composed_then_all_rules_have_unique_registration_ids() {
    let module = RuleModule::new();
    let ids = module.active_rule_ids();
    assert_eq!(ids.len(), 130);
    assert_eq!(ids.iter().copied().collect::<BTreeSet<_>>().len(), 130);
    assert!(
        ids.iter()
            .all(|id| crate::catalogue::util::find_catalogue_entry(id).is_some())
    );
    assert_eq!(module.rules().len(), ids.len());
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
