#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetiredRule {
    pub(crate) id: &'static str,
    pub(crate) replacement: Option<&'static str>,
}
