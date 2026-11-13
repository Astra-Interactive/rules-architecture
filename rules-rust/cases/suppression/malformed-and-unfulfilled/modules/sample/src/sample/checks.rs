#[cfg_attr(klibs_rules, expect(reason = "no rule id"))]
pub(crate) fn no_id() {}
#[cfg_attr(klibs_rules, expect(clippy::unwrap_used, reason = "a path"))]
pub(crate) fn path_id() {}
#[cfg_attr(klibs_rules, expect(AnonymousTuple))]
pub(crate) fn no_reason() -> (u8, u8) { (0, 0) }
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = 7))]
pub(crate) fn number_reason() -> (u8, u8) { (0, 0) }
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "  "))]
pub(crate) fn blank_reason() -> (u8, u8) { (0, 0) }
#[cfg_attr(not(klibs_rules), expect(AnonymousTuple, reason = "another predicate"))]
pub(crate) fn other_predicate() {}
#[cfg_attr(klibs_rules, allow(AnonymousTuple))]
pub(crate) fn not_expect() {}
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #4: the first of two"))]
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #4: the second of two"))]
pub(crate) fn twice() -> (u8, u8) { (0, 0) }
#[cfg_attr(klibs_rules, expect(AnonymousTuple, AnonymousTuple, reason = "reported to klibs-rules as #5: one id twice"))]
pub(crate) fn id_twice() -> (u8, u8) { (0, 0) }
#[cfg_attr(klibs_rules, expect(NoSuchRule, reason = "reported to klibs-rules as #6"))]
pub(crate) fn unknown() {}
#[cfg_attr(klibs_rules, expect(KDocFiller, reason = "reported to klibs-rules as #7"))]
pub(crate) fn retired() {}
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #8: nothing to suppress"))]
pub(crate) fn unfulfilled() {}
#[cfg_attr(klibs_rules, expect(KindShape, reason = "reported to klibs-rules as #9: a later phase"))]
pub(crate) mod later_phase {}
#[cfg_attr(klibs_rules, expect(PlatformImport, reason = "reported to klibs-rules as #10: platform-words is empty"))]
pub(crate) fn platform_words_empty() {}
#[cfg_attr(klibs_rules, expect(ForbiddenComment, reason = "reported to klibs-rules as #11: not expectable"))]
pub(crate) fn not_expectable() {}
#[cfg_attr(klibs_rules, expect(AnonymousTuple, note = "an unknown argument"))]
pub(crate) fn unknown_argument() {}
#[cfg_attr(klibs_rules, expect(AnonymousTuple reason))]
pub(crate) fn unparsed_expect() {}
#[cfg_attr(klibs_rules expect(AnonymousTuple))]
pub(crate) fn unparsed_cfg_attr() {}
