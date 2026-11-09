#[cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
pub(crate) mod domain {
    pub(crate) struct Point(i32, i32);
}
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
pub(crate) mod shared {
    //! The shared module of the fixture.
    #![cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason = "reported to klibs-rules as #2: an inner form after //!"))]
    #![cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #3: a module position of a node rule"))]
    pub(crate) struct Pair(u8, u8);
}
#[cfg_attr(klibs_rules, expect(Unanalysable, reason = "reported to klibs-rules as #1: a fixture of the suppression semantics"))]
mod missing;
