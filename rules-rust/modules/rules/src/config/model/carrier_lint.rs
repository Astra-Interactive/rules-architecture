#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CarrierLint {
    pub(crate) group: &'static str,
    pub(crate) lint: &'static str,
    pub(crate) groups: &'static [&'static str],
}
