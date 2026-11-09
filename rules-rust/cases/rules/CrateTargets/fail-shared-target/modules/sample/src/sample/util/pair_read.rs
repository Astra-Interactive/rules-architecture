#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported as #1"))]
pub(crate) fn read_pair() -> (u8, u8) { (1, 2) }
