mod model { trait Contract {}
    struct Stateful { value: bool }
    impl Stateful { fn run() {} }
    struct Rule;
    impl Rule { fn run() {} }
    impl Contract for Rule {}
    struct Constants;
    impl Constants { const VALUE: bool = true; }
}
