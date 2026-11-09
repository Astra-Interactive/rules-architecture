mod util {
    enum LoadError { Missing }
    struct ParseError;
    #[derive(thiserror::Error)] enum Failure { Missing }
    enum PayResult { Paid }
}
mod errors { enum OldError { Missing } }
