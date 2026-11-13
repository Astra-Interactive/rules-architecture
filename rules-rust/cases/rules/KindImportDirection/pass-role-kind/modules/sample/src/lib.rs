mod api {
    pub(crate) trait Contract {}
}
mod model {
    pub(crate) struct Value;
}
mod util {
    pub(crate) fn compute_value() {}
}
mod limiter {
    use crate::model::Value;
    use crate::registry::VoteRegistry;
    use crate::util::compute_value;
    pub(crate) struct VoteLimiter;
}
mod registry {
    pub(crate) struct VoteRegistry;
}
mod usecase {
    use crate::api::Contract;
    use crate::limiter::VoteLimiter;
}
mod command {
    use crate::limiter::VoteLimiter;
}
mod di {
    use crate::limiter::VoteLimiter;
}
mod file {
    use crate::limiter::VoteLimiter;
}
mod stub {
    use crate::limiter::VoteLimiter;
}
