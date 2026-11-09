mod api {
    trait Missing {}
    trait Generic<T>: Send + Sync {}
    trait Other: Send + Sync + Clone {}
    trait Async: Send + Sync { async fn load(&self); }
}
struct Gateway;
impl api::Async for Gateway { async fn load(&self) {} }
