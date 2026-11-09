#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
extern crate alloc;
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
const LIMIT: u8 = 1;
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
static NAME: &str = "name";
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
pub(crate) union Bits {
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    whole: u32,
}
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
pub(crate) trait Alias = Clone;
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
use std::fmt::Write;
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
unsafe extern "C" {
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    fn native();
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    static NATIVE: u8;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    type Opaque;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    native_items!();
}
#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
macro_rules! sample_macro {
    () => {};
}
pub(crate) trait Shape {
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    const SIDES: u8;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    type Unit;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    fn area(&self) -> u8;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    trait_items!();
}
pub(crate) struct Square;
impl Square {
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    const SIDES: u8 = 4;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    fn side(#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))] self) -> u8 {
        1
    }
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    impl_items!();
}
impl Shape for Square {
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    type Unit = u8;
    const SIDES: u8 = 4;
    fn area(&self) -> u8 {
        1
    }
}
pub(crate) async fn statements(mut value: u8, future: std::future::Ready<u8>) -> u8 {
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    value = 2;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    future.await;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    {}
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    drop(value);
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    for _ in 0..1 {}
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    if true {}
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    loop {
        break;
    }
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    match value {
        _ => {}
    }
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    value.to_string();
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    value;
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    unsafe {}
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    while false {}
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    (value + 1);
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    statement_macro!();
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    return value;
}
pub(crate) fn tail() -> u8 {
    #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = "reported to klibs-rules as #20: a node kind"))]
    tail_macro!()
}
