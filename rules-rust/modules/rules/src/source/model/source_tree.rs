#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SourceTree {
    Src,
    Test,
    Fixture,
    TestsIt,
}
