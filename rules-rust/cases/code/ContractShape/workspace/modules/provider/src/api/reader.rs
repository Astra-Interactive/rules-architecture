#[cfg(feature = "native")]
pub trait Reader: Send + Sync {
    async fn load(&self);
}

#[cfg(not(feature = "native"))]
pub trait Reader: Send + Sync {
    fn load(&self);
}
