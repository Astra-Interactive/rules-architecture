fn run() {
    struct Value;
    enum Choice {}
    trait Work {}
    impl Value {}
    mod nested {}
    union Bits { value: u8 }
    consume(|| { struct Inner; });
    spawn(async { struct AsyncValue; });
}
