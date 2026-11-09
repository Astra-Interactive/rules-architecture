mod model {
    struct User { name: String }
    #[derive(Debug, Clone, PartialEq)] struct UserId(u64);
}
