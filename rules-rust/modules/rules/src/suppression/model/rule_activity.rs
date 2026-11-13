use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleActivity {
    pub(crate) active_rules: BTreeSet<String>,
    pub(crate) is_platform_words_empty: bool,
}
