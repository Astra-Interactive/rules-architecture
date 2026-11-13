use crate::model::{Claim, ClaimChunk};
impl Claim {
    pub(crate) fn covers(&self, chunk: &ClaimChunk) -> bool { true }
    pub(crate) fn with_chunk(self, chunk: ClaimChunk) -> Claim { self }
}
