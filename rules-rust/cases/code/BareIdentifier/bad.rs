mod model {
    struct User { id: String, group_id: Option<u64> }
    struct UserId { value: u64 }
    enum Choice { User { user_id: uuid::Uuid } }
}
