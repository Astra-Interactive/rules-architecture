use crate::model::{Claim, ClaimChunk};
pub(crate) fn is_covered(claim: &Claim, chunk: &ClaimChunk) -> bool { true }
pub(crate) fn add_chunk(claim: Claim, chunk: ClaimChunk) -> Claim { claim }
pub(crate) fn is_long(claim: &Claim, limit: u64) -> bool { true }
pub(crate) fn exceeds(claim: &Claim, limit: &Claim) -> bool { true }
