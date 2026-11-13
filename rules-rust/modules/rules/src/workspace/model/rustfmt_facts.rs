use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RustfmtFacts {
    NotQueried,
    Missing { binary: String },
    Queried { nightly_only_keys: BTreeSet<String> },
}
