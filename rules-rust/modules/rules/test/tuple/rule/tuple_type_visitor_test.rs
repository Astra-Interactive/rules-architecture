use crate::check::api::Rule;
use crate::test::check::usecase::read_case_context;
use crate::tuple::rule::AnonymousTuple;

#[test]
fn given_tuple_types_in_headers_and_bounds_when_checked_then_every_syntactic_position_is_checked() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[1].syntax = syn::parse_file("trait Contract<T = (u8, u8)>: Iterator<Item = (u8, u8)> where (u8, u8): Copy {} impl<T = (u8, u8)> Contract<(u8, u8)> for Wrapper<(u8, u8)> where (u8, u8): Copy { fn call(&self) -> (u8, u8) { let value: (u8, u8); } type Associated = (u8, u8); }").expect("valid syntax");
    assert_eq!(AnonymousTuple.check(&context).len(), 9);
    context.crates[0].files[1].syntax = syn::parse_file("trait Contract<T = ()>: Iterator<Item = u8> {} impl Contract<()> for Wrapper<u8> { fn call(&self) -> (u8, u8) { loop {} } }").expect("valid syntax");
    assert!(AnonymousTuple.check(&context).is_empty());
}
