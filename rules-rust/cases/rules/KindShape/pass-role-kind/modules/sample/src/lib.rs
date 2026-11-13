pub(crate) mod vote {
    pub(crate) mod limiter {
        pub(crate) use vote_cooldown_limiter::VoteCooldownLimiter;
        mod vote_cooldown_limiter;
    }
}
pub(crate) mod registry {
    pub(crate) use packet_registry::PacketRegistry;
    mod packet_registry;
}
