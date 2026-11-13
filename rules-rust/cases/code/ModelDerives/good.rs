mod model {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)] struct User { name: String }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct UserId(u64);
}
mod file { struct Row; }
