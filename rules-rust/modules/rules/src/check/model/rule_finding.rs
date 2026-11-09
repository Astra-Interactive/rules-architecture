use crate::check::model::Anchor;
use crate::check::model::Finding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleFinding {
    pub(crate) finding: Finding,
    pub(crate) anchor: Anchor,
}
