pub(crate) mod sample {
    pub(crate) struct Line {
        pub(crate) pair: (u8, u8),
    }
    pub(crate) struct Point(i32, i32);
    pub(crate) enum Shape {
        Segment(Point, Point),
    }
    pub(crate) type Pair = (u8, u8);
    pub(crate) const ORIGIN: (i32, i32) = (0, 0);
    pub(crate) trait Positioned {
        fn position(&self) -> (i32, i32);
    }
    pub(crate) fn split(items: Vec<(u8, u8)>) -> u8 {
        let first: (u8, u8) = items[0];
        let pairs = items.into_iter().collect::<Vec<(u8, u8)>>();
        first.0 + u8::try_from(pairs.len()).unwrap_or(0)
    }
    impl From<(u8, u8)> for Line {
        fn from(pair: (u8, u8)) -> Self {
            Line { pair }
        }
    }
}
