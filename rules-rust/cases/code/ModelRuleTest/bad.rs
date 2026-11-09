mod model { pub struct Money; } mod util { pub fn is_valid(_: &crate::model::Money) -> bool { true } } #[cfg(test)] mod test { mod util {
    #[tokio::test]
    async fn test() { let money = crate::model::Money; let valid = crate::util::is_valid(&money); assert!(valid); }
} }

