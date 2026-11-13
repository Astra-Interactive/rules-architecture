pub(crate) mod vote {
    pub(crate) mod limiter {
        pub(crate) use limit_items::CooldownLimiter;
        pub(crate) use vote_cooldown::VoteCooldown;
        mod limit_items;
        mod vote_cooldown;
    }
    pub(crate) mod parser {
        mod ballot_parser;
    }
}
