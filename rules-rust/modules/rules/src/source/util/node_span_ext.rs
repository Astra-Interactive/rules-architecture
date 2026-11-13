use crate::source::model::NodeSpan;
use crate::source::model::TextPosition;

impl NodeSpan {
    fn is_at_or_before(left: TextPosition, right: TextPosition) -> bool {
        left.line < right.line || (left.line == right.line && left.column <= right.column)
    }

    pub(crate) fn contains(&self, other: &NodeSpan) -> bool {
        Self::is_at_or_before(self.start, other.start) && Self::is_at_or_before(other.end, self.end)
    }
}
