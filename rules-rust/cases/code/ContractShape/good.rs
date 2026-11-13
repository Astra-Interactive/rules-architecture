mod api {
    trait Clock: Send + Sync { fn now(&self); }
    #[async_trait::async_trait]
    trait Gateway: Send + Sync { async fn load(&self); }
}
struct DefaultGateway;
#[async_trait::async_trait]
impl api::Gateway for DefaultGateway { async fn load(&self) {} }
