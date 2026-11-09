use renamed::Exported as Concrete;
struct Consumer;
impl Consumer {
    fn new() -> Self {
        let repository = Concrete::new();
        Self
    }
}
mod di {
    struct OwnerModule;
    impl OwnerModule { fn new() -> Self { let repository = crate::Concrete::new(); Self } }
}
