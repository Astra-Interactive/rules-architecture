mod api {
    use crate::limiter::VoteLimiter;
    pub(crate) trait Contract {}
}
mod model {
    use crate::limiter::VoteLimiter;
    pub(crate) struct Value;
}
mod util {
    use crate::limiter::VoteLimiter;
    pub(crate) fn compute_value() {}
}
mod limiter {
    use crate::api::Contract;
    use crate::usecase::PlaceVote;
    use crate::model::Value;
    use crate::util::compute_value;
    use crate::registry::VoteRegistry;
    pub(crate) struct VoteLimiter;
}
mod registry {
    pub(crate) struct VoteRegistry;
}
mod usecase {
    use crate::limiter::VoteLimiter;
    pub(crate) struct PlaceVote;
}
