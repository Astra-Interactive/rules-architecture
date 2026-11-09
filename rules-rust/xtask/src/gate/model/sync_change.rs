#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SyncChange {
    Changed {
        document: String,
        first_line: String,
    },
    Moved {
        document: String,
        first_line: String,
    },
    Unplaced {
        document: String,
        heading: String,
        line: String,
    },
    New {
        document: String,
        heading: String,
        first_line: String,
    },
}
