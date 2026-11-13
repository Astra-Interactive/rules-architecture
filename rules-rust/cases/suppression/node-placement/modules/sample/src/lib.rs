#[cfg_attr(klibs_rules, expect(Unanalysable, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
declare_sample!();
pub(crate) mod sample {
    #[cfg_attr(klibs_rules, expect(WildcardImport, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
    use std::collections::*;
    pub(crate) struct Line {
        #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
        pub(crate) pair: (u8, u8),
    }
    pub(crate) enum Shape {
        #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
        Segment(u8, u8),
    }
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
    pub(crate) type Pair = (u8, u8);
    pub(crate) trait Positioned {
        #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
        fn position(&self) -> (i32, i32);
    }
    pub(crate) fn split(#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))] items: Vec<(u8, u8)>) -> usize {
        #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
        let pairs: Vec<(u8, u8)> = items;
        #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
        {
            let _: Option<(u8, u8)> = None;
        }
        match pairs.len() {
            #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
            0 => {
                let _: Option<(u8, u8)> = None;
                0
            }
            count => count,
        }
    }
    pub(crate) struct Nested;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #21: the outer expectation"))]
    impl Nested {
        #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #22: the innermost expectation takes the finding"))]
        pub(crate) fn pair(&self) -> (u8, u8) {
            (0, 0)
        }
    }
    pub(crate) struct Counter;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
    impl Counter {
        pub(crate) fn first(&self) -> (u8, u8) {
            (0, 0)
        }
        pub(crate) fn second(&self, pair: (u8, u8)) -> u8 {
            pair.0
        }
    }
}
