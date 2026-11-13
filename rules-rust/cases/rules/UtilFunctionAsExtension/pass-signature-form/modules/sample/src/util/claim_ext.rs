use crate::model::{Claim, ClaimChunk};
impl Claim {
    pub(crate) fn is_long(&self, limit: u64) -> bool { true }
    pub(crate) fn with_chunk(self, chunk: ClaimChunk) -> Claim { self }
    pub(crate) fn overlaps(&self, other: &Claim) -> bool { true }
}
