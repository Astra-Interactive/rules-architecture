pub(crate) struct VoteCooldownLimiter {
    window: u32,
}

impl VoteCooldownLimiter {
    pub(crate) fn new(window: u32) -> Self {
        Self { window }
    }

    pub(crate) fn window(&self) -> u32 {
        self.window
    }
}
