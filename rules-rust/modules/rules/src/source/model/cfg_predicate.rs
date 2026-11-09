#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgPredicate {
    Atom { key: String, value: Option<String> },
    All(Vec<CfgPredicate>),
    Any(Vec<CfgPredicate>),
    Not(Box<CfgPredicate>),
}
