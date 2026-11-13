mod model {
    struct UserId(u64);
    struct User { id: UserId, value: String }
    #[derive(serde::Serialize)] struct Wire { id: UserId, value: String }
    #[derive(serde::Deserialize)] struct WireId(String);
    enum StatusId { Empty }
}
mod file { struct Row { id: String } }
